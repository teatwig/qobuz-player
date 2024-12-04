import type { Track } from './Track';
import type { Playlist } from './Playlist.ts';

export function parsePlaylist(playlist: Playlist): ParsedPlaylist {
	const tracks: Track[] = Object.values(playlist.tracks)
		.filter((track) => track)
		.map((track) => track!);

	return {
		title: playlist.title,
		durationSeconds: playlist.durationSeconds,
		tracksCount: playlist.tracksCount,
		id: playlist.id,
		coverArt: playlist.coverArt,
		tracks: tracks
	};
}

export type ParsedPlaylist = Omit<Playlist, 'tracks'> & {
	tracks: Track[];
};
