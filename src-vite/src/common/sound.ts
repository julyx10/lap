/**
 * Short UI feedback sounds, synthesized with the Web Audio API (no audio assets).
 */

export type UiSound = 'next' | 'close';

// frequency (Hz) and duration (s) of each sound
const SOUNDS: Record<UiSound, { frequency: number; duration: number }> = {
  next: { frequency: 880, duration: 0.06 },
  close: { frequency: 440, duration: 0.09 },
};

const VOLUME = 0.15;

let audioContext: AudioContext | null = null;

export function playUiSound(sound: UiSound) {
  try {
    audioContext ??= new AudioContext();
    if (audioContext.state === 'suspended') {
      void audioContext.resume();
    }

    const { frequency, duration } = SOUNDS[sound];
    const now = audioContext.currentTime;
    const oscillator = audioContext.createOscillator();
    const gain = audioContext.createGain();

    oscillator.type = 'sine';
    oscillator.frequency.value = frequency;
    // quick attack and exponential decay to avoid clicks
    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.exponentialRampToValueAtTime(VOLUME, now + 0.005);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);

    oscillator.connect(gain).connect(audioContext.destination);
    oscillator.start(now);
    oscillator.stop(now + duration);
  } catch (error) {
    console.warn('playUiSound failed:', error);
  }
}
