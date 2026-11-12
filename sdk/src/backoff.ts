/** Doubling waits between retries, from `first` up to `max`. */
export class Backoff {
	private attempt = 0;

	constructor(
		private readonly first: number,
		private readonly max: number,
	) {}

	next(): number {
		const wait = Math.min(this.first * 2 ** this.attempt, this.max);
		this.attempt += 1;
		return wait;
	}

	reset(): void {
		this.attempt = 0;
	}
}
