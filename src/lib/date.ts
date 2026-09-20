import dayjs from 'dayjs';
import duration from 'dayjs/plugin/duration';
import relativeTime from 'dayjs/plugin/relativeTime';
import updateLocale from 'dayjs/plugin/updateLocale';

dayjs.extend(duration);
dayjs.extend(relativeTime);
dayjs.extend(updateLocale);

dayjs.updateLocale('en', {
	relativeTime: {
		s: (number: number) => `${number} seconds`,
	},
});

export function timeAgo(timestamp: number, now: number): string {
	const diffInMs = now - timestamp;

	if (diffInMs <= 4000 && diffInMs >= -1000) {
		return 'just now';
	}

	return dayjs(timestamp).from(dayjs(now));
}

export function formatTimeLeft(target: Date | undefined, now: number): string {
	if (!target) return 'Unavailable';

	const totalSeconds = Math.max(0, Math.ceil(dayjs(target).diff(dayjs(now), 'second', true)));
	const remaining = dayjs.duration(totalSeconds, 'seconds');
	const days = Math.floor(remaining.asDays());
	const hours = remaining.hours();
	const minutes = remaining.minutes();
	const seconds = remaining.seconds();

	if (days > 0) return `${days}d ${hours}h`;
	if (hours > 0) return `${hours}h ${minutes}m`;
	return `${minutes}m ${seconds}s`;
}
