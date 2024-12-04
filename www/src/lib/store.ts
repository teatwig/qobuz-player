import { Controls } from './controls';
import { derived, writable, type Writable } from 'svelte/store';
import type { TrackListValue } from './bindings/TrackListValue';
import type { Album } from './bindings/Album';

export const controls = writable<Controls>();

export const currentStatus: Writable<'Stopped' | 'Playing' | 'Paused'> = writable('Stopped');
export const connected = writable(false);
export const isBuffering = writable(false);
export const isLoading = writable(false);

export const position = writable(0);
export const currentTrackList: Writable<TrackListValue | null> = writable(null);

export const currentTrack = derived(currentTrackList, (list) => {
	return list?.queue.find((l) => l.status === 'Playing');
});

export const queue = derived(currentTrackList, (v) => {
	return v?.queue || [];
});

export const numOfTracks = derived(queue, (q) => {
	return q.length;
});

export const listType = derived(currentTrackList, (v) => {
	return v?.list_type;
});

export const coverImage = derived([currentTrackList, currentTrack], ([tl, c]) => {
	if (tl) {
		switch (tl.list_type) {
			case 'Album':
				return tl?.album?.coverArt ?? null;
			case 'Playlist':
				return tl?.playlist?.coverArt ?? null;
			case 'Track':
				return c?.album?.coverArt ?? null;
			case 'Unknown':
				return null;
		}
	}

	return null;
});

export const entityTitle = derived([currentTrackList, currentTrack], ([tl, c]) => {
	if (tl) {
		switch (tl.list_type) {
			case 'Album':
				return tl?.album?.title ?? null;
			case 'Playlist':
				return tl?.playlist?.title ?? null;
			case 'Track':
				return c?.album?.title ?? null;
			case 'Unknown':
				return null;
		}
	}

	return null;
});

export const secsToTimecode = (secs: number) => {
	const minutes = Math.floor(secs / 60);
	const seconds = secs - minutes * 60;

	return `${minutes.toString(10).padStart(2, '0')}:${seconds.toString(10).padStart(2, '0')}`;
};

export const positionString = derived(position, (p) => {
	const positionMinutes = Math.floor(p / 60);
	const positionSeconds = p - positionMinutes * 60;

	return `${positionMinutes.toString(10).padStart(2, '0')}:${positionSeconds.toString(10).padStart(2, '0')}`;
});

export const durationString = derived(currentTrack, (d) => {
	if (d === undefined) {
		return '00:00';
	}

	const durationMinutes = Math.floor(d.durationSeconds / 60);
	const durationSeconds = d.durationSeconds - durationMinutes * 60;

	return `${durationMinutes.toString(10).padStart(2, '0')}:${durationSeconds.toString(10).padStart(2, '0')}`;
});

export const artistAlbums = writable<{ id: number | null; albums: Album[] }>({
	id: null,
	albums: []
});
