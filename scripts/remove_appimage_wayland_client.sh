#!/usr/bin/env bash
# Post-process Lap AppImages built by Tauri/linuxdeploy (issues #270 and #271).
# Each input AppImage is extracted, patched, and replaced with a repacked image.
#
# Packaging policy:
# - Default the GTK launch hook to wayland,x11; preserve user-selected backends.
# - Use host Wayland, GStreamer, GLib, and selected support libraries to avoid
#   mixing bundled older libraries with newer host drivers and media plugins.
# - Disable bundled GStreamer plugin paths so host plugin discovery can work.
# - Optionally embed update information and generate a .zsync file.
#
# Assumes bundleMediaFramework is disabled; this script does not check that
# setting. GTK/WebKitGTK remain bundled, so hosts must provide libraries that
# satisfy their dependencies. Validate final images on supported distributions.
# These packaging changes do not apply to .deb packages.
#
# Usage:
#   remove_appimage_wayland_client.sh <appimage-directory>
#
# Optional environment variables:
#   UPDATE_INFO  appimagetool `-u` value (e.g. "zsync|https://.../{name}.zsync")
#   ZSYNC_URL    full-download URL written into the generated .zsync file
#   VERSION      version string used to expand UPDATE_INFO's {name_star}
# Both URL settings expand {name} to the AppImage basename. UPDATE_INFO also
# expands {name_star} when VERSION is set, replacing its first occurrence in
# the basename with * for gh-releases-zsync matching.
#
# Repacking also replaces the embedded AppImage runtime; test the repacked
# artifact, not only the extracted application.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <appimage-directory>" >&2
  exit 64
fi

APPIMAGE_DIR="$1"
UPDATE_INFO="${UPDATE_INFO:-}"
ZSYNC_URL="${ZSYNC_URL:-}"
VERSION="${VERSION:-}"

if [ ! -d "$APPIMAGE_DIR" ]; then
  echo "AppImage directory does not exist: $APPIMAGE_DIR" >&2
  exit 66
fi
# Keep input paths valid after the extraction subshell changes directory.
APPIMAGE_DIR="$(cd "$APPIMAGE_DIR" && pwd)"

case "$(uname -m)" in
  x86_64|aarch64) APPIMAGE_ARCH="$(uname -m)" ;;
  *)
    echo "Unsupported AppImage architecture: $(uname -m)" >&2
    exit 69
    ;;
esac

WORK_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

APPIMAGETOOL="$WORK_DIR/appimagetool.AppImage"
curl --fail --location --silent --show-error \
  --output "$APPIMAGETOOL" \
  "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${APPIMAGE_ARCH}.AppImage"
chmod +x "$APPIMAGETOOL"
# Run appimagetool without requiring a FUSE mount on the build runner.
export APPIMAGE_EXTRACT_AND_RUN=1

shopt -s nullglob
APPIMAGES=("$APPIMAGE_DIR"/*.AppImage)
if [ "${#APPIMAGES[@]}" -eq 0 ]; then
  echo "No AppImages found in: $APPIMAGE_DIR" >&2
  exit 66
fi

for appimage in "${APPIMAGES[@]}"; do
  image_name="$(basename "$appimage")"
  image_work_dir="$WORK_DIR/${image_name}.work"
  mkdir "$image_work_dir"

  echo "==> Extracting ${image_name}"
  (
    cd "$image_work_dir"
    "$appimage" --appimage-extract >/dev/null
  )

  # The bundled Wayland client caused EGL initialization failures with host
  # graphics libraries (#270/#271). Remove every copy/symlink across build
  # layouts. Its absence is fine — there is simply nothing to strip — so log and
  # continue with the rest of the post-processing instead of failing the build.
  wayland_clients=()
  while IFS= read -r -d '' wayland_client; do
    wayland_clients+=("$wayland_client")
  done < <(find "$image_work_dir/squashfs-root" -name 'libwayland-client.so*' -print0)
  if [ "${#wayland_clients[@]}" -gt 0 ]; then
    echo "==> Removing bundled libwayland-client from ${image_name}"
    rm -f "${wayland_clients[@]}"
  else
    echo "No bundled libwayland-client found in ${image_name}; nothing to remove" >&2
  fi

  # Apply the default before the application starts, only for AppImages.
  # ${VAR-default} preserves explicit values (including empty), allowing users
  # to select X11. Upstream linuxdeploy-plugin-gtk now ships the assignment
  # commented out ("# export GDK_BACKEND=x11 # ... Wayland"), so also match an
  # optional leading '#' and replace the whole line with our active default.
  # Require exactly one match so further upstream hook changes get reviewed.
  gtk_hook="$image_work_dir/squashfs-root/apprun-hooks/linuxdeploy-plugin-gtk.sh"
  if [ ! -f "$gtk_hook" ]; then
    echo "GTK launch hook not found in ${image_name}" >&2
    exit 1
  fi
  echo "==> Configuring AppImage GTK backend default in ${image_name}"
  LC_ALL=C perl -0777 -i -pe '
    BEGIN { $replacement = q{export GDK_BACKEND="${GDK_BACKEND-wayland,x11}"}; }
    $count = s/^[ \t]*#?[ \t]*export[ \t]+GDK_BACKEND=[^\r\n]*/$replacement/gm;
    die "Expected exactly one GDK_BACKEND assignment in GTK hook; review upstream changes\n"
      unless $count == 1;
  ' "$gtk_hook"
  bash -n "$gtk_hook"

  # AppRun.wrapped sets both GST_PLUGIN_SYSTEM_PATH_1_0 and
  # GST_PLUGIN_SYSTEM_PATH to bundle directories even without media plugins
  # (tauri-apps/tauri#15665). Neutralize both: disabling only the versioned name
  # leaves the unversioned override blocking default host plugin discovery.
#
  # AppRun.wrapped is ELF, not a shell script. Rename variables with equal-length
  # byte replacements to preserve offsets; deleting matching lines corrupts it.
  apprun_wrapped="$image_work_dir/squashfs-root/AppRun.wrapped"
  if [ ! -f "$apprun_wrapped" ]; then
    echo "AppRun.wrapped not found in ${image_name}" >&2
    exit 1
  fi
  apprun_size_before="$(wc -c < "$apprun_wrapped")"
  # Handle the longer name first: the unversioned name is its prefix.
  gst_path_replacements=(
    'GST_PLUGIN_SYSTEM_PATH_1_0:LAP_IGNORE_GST_PLUGIN_PATH'
    'GST_PLUGIN_SYSTEM_PATH:LAP_IGNORE_2ND_GST_PTH'
  )
  for gst_path_replacement in "${gst_path_replacements[@]}"; do
    gst_path_var="${gst_path_replacement%%:*}"
    disabled_gst_path_var="${gst_path_replacement#*:}"
    if [ "${#gst_path_var}" -ne "${#disabled_gst_path_var}" ]; then
      echo "Internal error: replacement GStreamer variable has a different length" >&2
      exit 1
    fi
    if LC_ALL=C grep -aq "$gst_path_var" "$apprun_wrapped"; then
      echo "==> Disabling ${gst_path_var} override in ${image_name}"
      LC_ALL=C perl -0777 -i -pe "s/${gst_path_var}/${disabled_gst_path_var}/g" "$apprun_wrapped"
      if LC_ALL=C grep -aq "$gst_path_var" "$apprun_wrapped" || \
        ! LC_ALL=C grep -aq "$disabled_gst_path_var" "$apprun_wrapped"; then
        echo "Failed to disable GStreamer plugin path override in ${image_name}" >&2
        exit 1
      fi
    else
      # Already patched or no longer emitted upstream: no replacement needed.
      echo "${gst_path_var} override not found in ${image_name}; nothing to do" >&2
    fi
  done
  if [ "$(wc -c < "$apprun_wrapped")" -ne "$apprun_size_before" ] || \
    LC_ALL=C grep -aFq 'GST_PLUGIN_SYSTEM_PATH' "$apprun_wrapped"; then
    echo "GStreamer patch changed binary size or left a plugin path override in ${image_name}" >&2
    exit 1
  fi
  # linuxdeploy copies WebKitGTK's linked media libraries without the matching
  # runtime-loaded plugins/scanner. Remove that partial stack and selected
  # shared dependencies: #271 exposed successive GStreamer, GLib, libmount,
  # and PCRE2 mismatches, and #311 showed host libmount requiring a newer
  # libsystemd than the one bundled. This list follows those reports and
  # tauri#15665; it does not guarantee compatibility with every host or cover
  # all dependencies. The scan below handles regular files directly under the
  # current usr/lib layout; revisit it if the bundler changes library paths or
  # symlink layouts.
  host_runtime_lib_patterns=(
    'libwayland-cursor.so*'
    'libwayland-egl.so*'
    'libwayland-server.so*'
    'libgst*.so*'
    'libgstreamer-*.so*'
    'liborc-*.so*'
    'libglib-2.0.so*'
    'libgobject-2.0.so*'
    'libgio-2.0.so*'
    'libgmodule-2.0.so*'
    'libgthread-2.0.so*'
    'libffi.so*'
    'libmount.so*'
    'libblkid.so*'
    'libselinux.so*'
    'libsystemd.so*'
    'libudev.so*'
    'libpcre2-8.so*'
    'libzstd.so*'
    'libelf.so*'
  )
  host_runtime_libs=()
  for host_runtime_lib_pattern in "${host_runtime_lib_patterns[@]}"; do
    while IFS= read -r -d '' host_runtime_lib; do
      host_runtime_libs+=("$host_runtime_lib")
    done < <(find "$image_work_dir/squashfs-root/usr/lib" -maxdepth 1 -type f -name "$host_runtime_lib_pattern" -print0)
  done
  if [ "${#host_runtime_libs[@]}" -gt 0 ]; then
    echo "==> Removing partial bundled host runtime libraries from ${image_name}"
    rm -f "${host_runtime_libs[@]}"
  else
    echo "No bundled host runtime libraries found in ${image_name}; nothing to do" >&2
  fi

  repack_args=(--no-appstream)
  if [ -n "$UPDATE_INFO" ]; then
    update_info="${UPDATE_INFO//\{name\}/$image_name}"
    if [ -n "$VERSION" ]; then
      name_star="${image_name/${VERSION}/*}"
      update_info="${update_info//\{name_star\}/$name_star}"
    fi
    repack_args+=(-u "$update_info")
  fi

  echo "==> Repacking ${image_name}"
  # Extracted modes may be owner-only; the final AppDir must work for all users.
  chmod -R a+rX "$image_work_dir/squashfs-root"
  chmod a+rx "$image_work_dir/squashfs-root/AppRun"
  replacement="$image_work_dir/${image_name}.new"
  "$APPIMAGETOOL" "${repack_args[@]}" \
    "$image_work_dir/squashfs-root" "$replacement" >/dev/null
  chmod +x "$replacement"
  mv "$replacement" "$appimage"

  # Generate delta metadata from the final bytes, after patching and repacking.
  if [ -n "$ZSYNC_URL" ]; then
    zsync_url="${ZSYNC_URL//\{name\}/$image_name}"
    echo "==> Generating .zsync for ${image_name}"
    zsyncmake -u "$zsync_url" -o "$appimage.zsync" "$appimage"
  fi
done
