import { getCurrentWindow } from '@tauri-apps/api/window';

/** Make noninteractive title-bar content a native window drag surface. */
export function windowDrag(node: HTMLElement) {
	function onMouseDown(event: MouseEvent) {
		if (event.button !== 0 || !(event.target instanceof Element)) return;
		if (event.target.closest('button, a, input, textarea, select, [role="button"], [contenteditable="true"]')) return;
		void getCurrentWindow().startDragging().catch((error) => {
			console.error('Could not drag window:', error);
		});
	}

	node.addEventListener('mousedown', onMouseDown);
	return { destroy: () => node.removeEventListener('mousedown', onMouseDown) };
}
