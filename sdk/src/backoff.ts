/**
 * Doubling waits between retries, from `first` up to `max`. Each wait is cut by up to half
 * at random, so clients that failed together don't all retry at the same instant.
 */
export class Backoff {
	private attempt = 0;

	constructor(
		private readonly first: number,
		private readonly max: number,
		private readonly random: () => number = Math.random,
	) {}

	next(): number {
		const wait = Math.min(this.first * 2 ** this.attempt, this.max);
		this.attempt += 1;
		return wait * (1 - this.random() / 2);
	}

	reset(): void {
		this.attempt = 0;
	}
}
