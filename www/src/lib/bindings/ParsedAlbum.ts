import type { Album } from './Album';
import type { Track } from './Track';

export function parseAlbum(album: Album): ParsedAlbum {
	const tracks: Track[] = Object.values(album.tracks)
		.filter((track) => track)
		.map((track) => track!);

	return {
		id: album.id,
		title: album.title,
		artist: album.artist,
		releaseYear: album.releaseYear,
		hiresAvailable: album.hiresAvailable,
		explicit: album.explicit,
		totalTracks: album.totalTracks,
		tracks: tracks,
		available: album.available,
		coverArt: album.coverArt,
		coverArtSmall: album.coverArtSmall
	};
}

export type ParsedAlbum = Omit<Album, 'tracks'> & {
	tracks: Track[];
};
