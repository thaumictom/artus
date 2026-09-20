import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import updateLocale from 'dayjs/plugin/updateLocale';

dayjs.extend(relativeTime);
dayjs.extend(updateLocale);

dayjs.updateLocale('en', {
	relativeTime: {
		s: (number: number) => `${number} seconds`,
	}
});

export function timeAgo(timestamp: number, now: number): string {
	const diffInMs = now - timestamp;

	if (diffInMs <= 4000 && diffInMs >= -1000) {
		return 'just now';
	}

	return dayjs(timestamp).from(dayjs(now));
}
