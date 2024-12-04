import type { Action } from './bindings/Action';
import type { Album } from './bindings/Album';
import type { Artist } from './bindings/Artist';
import type { Favorites } from './bindings/Favorites';
import type { Playlist } from './bindings/Playlist';
import { parsePlaylist } from './bindings/ParsedPlaylist';
import type { SearchResults } from './bindings/SearchResults';
import {
	connected,
	isBuffering,
	isLoading,
	position,
	currentStatus,
	currentTrackList,
	artistAlbums
} from './store';
import { parseAlbum } from './bindings/ParsedAlbum';

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

		this.connect();
	}

	connect() {
		this.ws = new WebSocket(`${this.webSocketProtocol}//${this.host}/ws`);
		this.ws.onopen = () => {
			connected.set(true);
		};

		this.ws.onclose = () => this.reconnect();

		this.ws.onmessage = (message) => {
			const json = JSON.parse(message.data);

			switch (true) {
				case Object.hasOwn(json, 'buffering'):
					isBuffering.set(json.buffering.is_buffering);
					break;
				case Object.hasOwn(json, 'loading'):
					isLoading.set(json.loading.is_loading);
					break;
				case Object.hasOwn(json, 'position'):
					position.set(json.position.clock);
					break;
				case Object.hasOwn(json, 'status'):
					currentStatus.set(json.status.status);
					break;
				case Object.hasOwn(json, 'currentTrackList'):
					currentTrackList.set(json.currentTrackList?.list);
					break;
				case Object.hasOwn(json, 'artistAlbums'):
					artistAlbums.set(json.artistAlbums);
					break;
			}
		};

		this.ws.onerror = () => {
			this.ws?.close();
			this.reconnect();
		};
	}

	reconnect() {
		connected.set(false);

		setTimeout(() => {
			this.connect();
		}, 1000);
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

	playPlaylist(playlist_id: number) {
		this.send({ playPlaylist: { playlist_id: playlist_id as unknown as bigint } });
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

	async favorites() {
		const url = `${location.protocol}//${this.host}/api/favorites`;
		const result = await fetch(url).then((res) => {
			if (!res.ok) {
				return;
			}
			return res.json() as unknown as Favorites;
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

	async set_favorite(
		id: string,
		add: boolean,
		type: 'album' | 'artist' | 'playlist'
	): Promise<boolean> {
		const url = `${location.protocol}//${this.host}/api/favorite/${type}/${id}`;
		await fetch(url, { method: add ? 'POST' : 'DELETE', body: 'body' });
		return add;
	}

	async favoritePlaylists() {
		const url = `${location.protocol}//${this.host}/api/favorite-playlists`;
		const result = await fetch(url).then((res) => {
			if (!res.ok) {
				return;
			}
			return res.json() as unknown as Playlist[];
		});

		return result ?? [];
	}

	async artist(artistId: number | undefined) {
		if (artistId === undefined) {
			return undefined;
		}

		const url = `${location.protocol}//${this.host}/api/artists/${artistId}`;
		const result = await fetch(url).then((res) => {
			if (!res.ok) {
				return;
			}
			return res.json() as unknown as Artist;
		});

		return result ?? undefined;
	}

	async album(id: string | undefined) {
		if (id === undefined) {
			return undefined;
		}

		const url = `${location.protocol}//${this.host}/api/albums/${id}`;
		const result = await fetch(url).then((res) => {
			if (!res.ok) {
				return;
			}
			return res.json() as unknown as Album;
		});

		return result ? parseAlbum(result) : undefined;
	}

	async artistAlbumReleases(artistId: number | undefined) {
		if (artistId === undefined) {
			return undefined;
		}

		const url = `${location.protocol}//${this.host}/api/artists/${artistId}/releases`;
		const result = await fetch(url).then((res) => {
			if (!res.ok) {
				return;
			}
			return res.json() as unknown as Album[];
		});

		return result ?? undefined;
	}

	async playlist(id: number | undefined) {
		if (id === undefined) {
			return undefined;
		}

		const url = `${location.protocol}//${this.host}/api/playlist/${id}`;
		const result = await fetch(url).then((res) => {
			if (!res.ok) {
				return;
			}
			return res.json() as unknown as Playlist;
		});

		return result ? parsePlaylist(result) : undefined;
	}

	async isFavoriteAlbum(id: string): Promise<boolean> {
		const favorites = await this.favorites();
		return favorites.albums.map((album) => album.id).includes(id);
	}

	async isFavoriteArtist(id: number): Promise<boolean> {
		const favorites = await this.favorites();
		return favorites.artists.map((artist) => artist.id).includes(id);
	}

	async isFavoritePlaylist(id: number): Promise<boolean> {
		const favorites = await this.favoritePlaylists();
		return favorites.map((favorite) => favorite.id).includes(id);
	}
}
