import type { Config } from 'tailwindcss';
import safeArea from 'tailwindcss-safe-area';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			animation: {
				delay: 'delay 1s ease-in-out'
			},

			keyframes: {
				delay: {
					'0%': { opacity: '0' },
					'90%': { opacity: '0' },
					'100%': { opacity: '1' }
				}
			}
		}
	},
	plugins: [safeArea],
	future: {
		hoverOnlyWhenSupported: true
	}
} satisfies Config;
