<template>
  <canvas ref="canvas" class="fireworks-canvas"></canvas>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue'

const canvas = ref(null)
let ctx = null
let animationId = null
let particles = []
let rockets = []

const colors = [
  '#FF0000', // Red
  '#FF7F00', // Orange
  '#FFFF00', // Yellow
  '#00FF00', // Green
  '#0000FF', // Blue
  '#4B0082', // Indigo
  '#9400D3', // Violet
  '#FFD700', // Gold
  '#FF69B4', // HotPink
]

class Rocket {
  constructor(x, y) {
    this.x = x
    this.startX = x
    this.y = window.innerHeight
    this.targetY = y
    // Slower speed
    this.speed = Math.random() * 3 + 4 
    this.color = colors[Math.floor(Math.random() * colors.length)]
    this.trail = []
    
    // Wobble parameters
    this.wobbleFreq = Math.random() * 0.05 + 0.02
    this.wobbleAmp = Math.random() * 4 + 2
    this.timer = 0
  }

  update() {
    this.y -= this.speed
    this.timer++
    
    // Wobble effect: sine wave on X axis
    this.x = this.startX + Math.sin(this.timer * this.wobbleFreq * 10) * (Math.random() * 1.5 + 0.5) // Reduced amplitude
    
    // Create trail (history of positions)
    this.trail.push({ x: this.x, y: this.y })
    if (this.trail.length > 8) this.trail.shift()

    // Check if reached target (approximate, since we might overshoot or wobble)
    if (this.y <= this.targetY) {
      this.explode()
      return false // Remove rocket
    }
    return true // Keep rocket
  }

  draw() {
    ctx.save()
    ctx.beginPath()
    // Draw trail line
    if (this.trail.length > 0) {
      ctx.moveTo(this.trail[0].x, this.trail[0].y)
      for (let i = 1; i < this.trail.length; i++) {
        ctx.lineTo(this.trail[i].x, this.trail[i].y)
      }
    }
    ctx.strokeStyle = this.color
    ctx.lineWidth = 2
    ctx.stroke()
    
    // Head
    ctx.beginPath()
    ctx.arc(this.x, this.y, 3, 0, Math.PI * 2)
    ctx.fillStyle = '#fff'
    ctx.fill()
    ctx.restore()
  }

  explode() {
    const particleCount = 80 // More particles
    // Explosion sound effect (optional/visual only here)
    for (let i = 0; i < particleCount; i++) {
        // Random angle and velocity
        const angle = Math.random() * Math.PI * 2
        // Double the explosion distance (velocity)
        const velocity = Math.random() * 6 + 2 
        // Random color for each particle
        const particleColor = colors[Math.floor(Math.random() * colors.length)]
        particles.push(new Particle(this.x, this.y, Math.cos(angle) * velocity, Math.sin(angle) * velocity, particleColor))
    }
  }
}

class Particle {
  constructor(x, y, dx, dy, color) {
    this.x = x
    this.y = y
    this.dx = dx
    this.dy = dy
    this.color = color
    // Halve the particle size
    this.radius = Math.random() * 1 + 0.5
    this.life = 150 // Longer life
    this.gravity = 0.02 // Reduced gravity
    this.friction = 0.96 // Slow down over time
    this.history = []
  }

  draw() {
    ctx.save()
    ctx.beginPath()
    // Draw particle trail
    if (this.history.length > 0) {
        ctx.moveTo(this.history[0].x, this.history[0].y)
        for(let i = 1; i < this.history.length; i++) {
            ctx.lineTo(this.history[i].x, this.history[i].y)
        }
    }
    
    ctx.strokeStyle = this.color
    ctx.globalAlpha = this.life / 150
    ctx.lineWidth = this.radius
    ctx.stroke()
    
    // Draw particle head
    ctx.beginPath()
    ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2)
    ctx.fillStyle = this.color
    ctx.fill()
    
    ctx.restore()
  }

  update() {
    // Add current position to history for trail
    this.history.push({ x: this.x, y: this.y })
    if (this.history.length > 30) this.history.shift() // Longer trail

    this.dy += this.gravity
    this.dx *= this.friction
    this.dy *= this.friction
    
    this.x += this.dx
    this.y += this.dy
    
    this.life -= 1 // Slower decay
    this.draw()
  }
}

const resize = () => {
  if (canvas.value) {
    canvas.value.width = window.innerWidth
    canvas.value.height = window.innerHeight
  }
}

const handleClick = (e) => {
  // Launch rocket towards the click Y height, horizontally centered on click X
  rockets.push(new Rocket(e.clientX, e.clientY))
}

const animate = () => {
  requestAnimationFrame(animate)
  if (!ctx) return
  
  // Clear with fade effect for global trail (optional, helps smoothen)
  ctx.globalCompositeOperation = 'destination-out'
  ctx.fillStyle = 'rgba(0, 0, 0, 0.15)' // Slow fade
  ctx.fillRect(0, 0, canvas.value.width, canvas.value.height)
  
  ctx.globalCompositeOperation = 'lighter' // Additive blending for glowing effect

  // Update Rockets
  rockets = rockets.filter(rocket => {
    rocket.draw()
    return rocket.update()
  })
  
  // Update Particles
  particles.forEach((particle, index) => {
    if (particle.life <= 0) {
      particles.splice(index, 1)
    } else {
      particle.update()
    }
  })
  
  // Reset composite operation
  ctx.globalCompositeOperation = 'source-over'
}

onMounted(() => {
  if (typeof window !== 'undefined') {
    ctx = canvas.value.getContext('2d')
    resize()
    window.addEventListener('resize', resize)
    window.addEventListener('click', handleClick)
    animate()
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('resize', resize)
    window.removeEventListener('click', handleClick)
    cancelAnimationFrame(animationId)
  }
})
</script>

<style scoped>
.fireworks-canvas {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  pointer-events: none;
  z-index: 0;
}
</style>
