import { expoOut } from 'svelte/easing';
import type { EasingFunction, TransitionConfig } from 'svelte/transition';

interface FlyAndScaleParams {
    delay?: number;
    duration?: number;
    easing?: EasingFunction;
    x?: number;
    y?: number;
    start?: number;
    opacity?: number;
}

export function flyAndScale(node: Element, {
    delay = 0,
    duration = 200,
    easing = expoOut,
    x = 0,
    y = -16,
    start = 0.98,
    opacity = 0
}: FlyAndScaleParams = {}): TransitionConfig {
    const style = getComputedStyle(node);
    const targetOpacity = +style.opacity;
    const transform = style.transform === 'none' ? '' : style.transform;

    return {
        delay,
        duration,
        easing,
        css: (t, u) => `
            transform: ${transform} translate(${x * u}px, ${y * u}px) scale(${start + (1 - start) * t});
            opacity: ${targetOpacity - (targetOpacity - opacity) * u};
        `
    };
}
