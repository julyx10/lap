<template>
  
  <div class="image-container" id="imageContainer">
    <img id="imageViewer" src="https://via.placeholder.com/800" alt="Image Viewer">
  </div>

</template>

<script>
    const image = document.getElementById('imageViewer');
    let isDragging = false;
    let scale = 1;
    let startX = 0;
    let startY = 0;
    let translateX = 0;
    let translateY = 0;
    let lastTranslateX = 0;
    let lastTranslateY = 0;

    // Mouse wheel event for zooming
    image.addEventListener('wheel', function(event) {
      event.preventDefault();

      const zoomSpeed = 0.2;
      const delta = event.deltaY < 0 ? zoomSpeed : -zoomSpeed;
      scale = Math.min(Math.max(0.5, scale + delta), 5); // Clamp zoom between 0.5 and 5
      updateTransform();
    });

    // Mouse down event to start dragging
    image.addEventListener('mousedown', function(event) {
      isDragging = true;
      startX = event.clientX - lastTranslateX;
      startY = event.clientY - lastTranslateY;
      image.style.cursor = 'grabbing';
    });

    // Mouse move event for dragging the image
    document.addEventListener('mousemove', function(event) {
      if (isDragging) {
        translateX = (event.clientX - startX) / scale;
        translateY = (event.clientY - startY) / scale;
        updateTransform();
      }
    });

    // Mouse up event to stop dragging
    document.addEventListener('mouseup', function() {
      if (isDragging) {
        lastTranslateX = translateX;
        lastTranslateY = translateY;
      }
      isDragging = false;
      image.style.cursor = 'grab';
    });

    // Function to update the image's transform (zoom and drag)
    function updateTransform() {
      image.style.transform = `scale(${scale}) translate(${translateX}px, ${translateY}px)`;
    }
</script>

<style scoped>
 body {
      margin: 0;
      padding: 0;
      overflow: hidden;
    }

    .image-container {
      display: flex;
      justify-content: center;
      align-items: center;
      height: 100vh;
      background-color: #2d3748;
      overflow: hidden;
    }

    img {
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
      cursor: grab;
      transition: transform 0.15s ease-in-out;
    }
</style>
