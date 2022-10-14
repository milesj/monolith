import { Action, ActionStatus } from '@moonrepo/types';
import { getDurationInMillis } from './time';

export function getIconForStatus(status: ActionStatus): string {
	switch (status) {
		case 'cached':
			return '🟪';
		// case 'cached-remote':
		// 	return '🟦';
		case 'failed':
		case 'failed-and-abort':
			return '🟥';
		case 'invalid':
			return '🟨';
		case 'passed':
			return '🟩';
		default:
			return '⬛️';
	}
}

export function hasFailed(status: ActionStatus): boolean {
	return status === 'failed' || status === 'failed-and-abort';
}

export function hasPassed(status: ActionStatus): boolean {
	return status === 'passed' || status === 'cached';
}

export function isFlaky(action: Action): boolean {
	if (!action.attempts || action.attempts.length === 0) {
		return false;
	}

	return hasPassed(action.status) && action.attempts.some((attempt) => hasFailed(attempt.status));
}

export function isSlow(action: Action, slowThreshold: number): boolean {
	if (!action.duration) {
		return false;
	}

	const millis = getDurationInMillis(action.duration);
	const threshold = slowThreshold * 1000; // In seconds

	return millis > threshold;
}
