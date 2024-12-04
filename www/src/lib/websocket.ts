import { derived, writable } from 'svelte/store';
import type { Writable } from 'svelte/store';
import type { Playlist } from './bindings/Playlist';
import type { TrackListValue } from './bindings/TrackListValue';
import type { Action } from './bindings/Action';
import type { Track } from './bindings/Track';
import type { Album } from './bindings/Album';
import type { SearchResults } from './bindings/SearchResults';

export const currentStatus: Writable<'Stopped' | 'Playing' | 'Paused'> = writable('Stopped');
export const connected = writable(false);
export const isBuffering = writable(false);
export const isLoading = writable(false);

export const userPlaylists: Writable<Playlist[]> = writable();

export const position = writable(0);
const currentTrackList: Writable<TrackListValue | null> = writable(null);

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
		return '0';
	}

	const durationMinutes = Math.floor(d.durationSeconds / 60);
	const durationSeconds = d.durationSeconds - durationMinutes * 60;

	return `${durationMinutes.toString(10).padStart(2, '0')}:${durationSeconds.toString(10).padStart(2, '0')}`;
});

export const artistAlbums = writable<{ id: number | null; albums: Array<Album> }>({
	id: null,
	albums: []
});

export const playlistTracks = writable<{ id: number | null; tracks: Array<Track> }>({
	id: null,
	tracks: []
});
export const playlistTitle = writable('');

export class Controls {
	dev: boolean;
	secure: boolean;
	webSocketProtocol: string;
	host: string;
	ws: WebSocket | undefined;

	constructor(dev: boolean) {
		this.dev = dev;
		this.secure = location.protocol === 'https:';
		this.webSocketProtocol = this.secure ? 'wss:' : 'ws:';
		this.host = dev ? 'localhost:9888' : window.location.host;

		this.playPause.bind(this);
		this.next.bind(this);
		this.previous.bind(this);
		this.close.bind(this);

		this.connect();
	}

	connect() {
		this.ws = new WebSocket(`${this.webSocketProtocol}//${this.host}/ws`);
		this.ws.onopen = () => {
			connected.set(true);
			this.fetchUserPlaylists();
		};

		this.ws.onclose = () => {
			connected.set(false);

			setTimeout(() => {
				this.connect();
			}, 1000);
		};

		this.ws.onmessage = (message) => {
			const json = JSON.parse(message.data);

			if (Object.hasOwn(json, 'buffering')) {
				isBuffering.set(json.buffering.is_buffering);
			} else if (Object.hasOwn(json, 'loading')) {
				isLoading.set(json.loading.is_loading);
			} else if (Object.hasOwn(json, 'position')) {
				position.set(json.position.clock);
			} else if (Object.hasOwn(json, 'status')) {
				currentStatus.set(json.status.status);
			} else if (Object.hasOwn(json, 'currentTrackList')) {
				currentTrackList.set(json.currentTrackList?.list);
			} else if (Object.hasOwn(json, 'artistAlbums')) {
				artistAlbums.set(json.artistAlbums);
			} else if (Object.hasOwn(json, 'playlistTracks')) {
				playlistTracks.set(json.playlistTracks);
			} else if (Object.hasOwn(json, 'userPlaylists')) {
				userPlaylists.set(json.userPlaylists);
			}
		};

		this.ws.onerror = () => {
			this.ws?.close();
		};
	}

	close() {
		this.ws?.close();
	}

	send(action: Action) {
		this.ws?.send(JSON.stringify(action));
	}

	playPause() {
		this.send('playPause');
	}

	next() {
		this.send('next');
	}

	previous() {
		this.send('previous');
	}

	skipTo(num: number) {
		this.send({ skipTo: { num } });
	}

	playAlbum(album_id: string) {
		this.send({ playAlbum: { album_id } });
	}

	playTrack(track_id: number) {
		this.send({ playTrack: { track_id } });
	}

	playPlaylist(playlist_id: bigint) {
		this.send({ playPlaylist: { playlist_id } });
	}

	fetchArtistAlbums(artist_id: number) {
		this.send({ fetchArtistAlbums: { artist_id } });
	}

	fetchPlaylistTracks(playlist_id: number) {
		this.send({ fetchPlaylistTracks: { playlist_id: BigInt(playlist_id) } });
	}

	fetchUserPlaylists() {
		this.send('fetchUserPlaylists');
	}

	async search(query: string, abortController: AbortController) {
		const url = `${location.protocol}//${this.host}/api/search?query=${query}`;
		const result = await fetch(url, abortController).then((res) => {
			if (!res.ok) {
				return;
			}
			return res.json() as unknown as SearchResults;
		});

		return (
			result ?? {
				query: '',
				albums: [],
				tracks: [],
				artists: [],
				playlists: []
			}
		);
	}
}
