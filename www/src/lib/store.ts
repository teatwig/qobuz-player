import { Controls } from '$lib/websocket';
import { writable } from 'svelte/store';

export const controls = writable<Controls>();
