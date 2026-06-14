import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildFallbackPrompt,
  buildReversePromptInstruction,
} from './reversePrompt.js';

test('buildReversePromptInstruction asks for a reusable prompt only', () => {
  const instruction = buildReversePromptInstruction({
    tone: 'detailed',
    extraContext: 'Match a cinematic product photography style.',
  });

  assert.match(instruction, /reusable image-generation prompt/i);
  assert.match(instruction, /subject/i);
  assert.match(instruction, /lighting/i);
  assert.match(instruction, /one prompt only/i);
  assert.match(instruction, /cinematic product photography/i);
});

test('buildFallbackPrompt summarizes format, dimensions, color, and visual qualities', () => {
  const prompt = buildFallbackPrompt({
    fileName: 'studio-watch.png',
    width: 1600,
    height: 900,
    mimeType: 'image/png',
    sizeBytes: 2_400_000,
    hasAlpha: true,
    averageBrightness: 0.72,
    averageSaturation: 0.68,
    contrast: 0.42,
    palette: ['#0f172a', '#f59e0b', '#f8fafc'],
  });

  assert.match(prompt, /wide 16:9/i);
  assert.match(prompt, /1600x900/i);
  assert.match(prompt, /transparent/i);
  assert.match(prompt, /bright/i);
  assert.match(prompt, /high-contrast/i);
  assert.match(prompt, /saturated/i);
  assert.match(prompt, /#0f172a, #f59e0b, #f8fafc/i);
  assert.doesNotMatch(prompt, /undefined|null|NaN/);
});
