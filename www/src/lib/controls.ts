import type { Action } from './bindings/Action';
import type { SearchResults } from './bindings/SearchResults';
import {
	connected,
	isBuffering,
	isLoading,
	position,
	currentStatus,
	currentTrackList,
	artistAlbums,
	playlistTracks,
	userPlaylists
} from './store';

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
		this.send({ fetchPlaylistTracks: { playlist_id: playlist_id as unknown as bigint } });
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
