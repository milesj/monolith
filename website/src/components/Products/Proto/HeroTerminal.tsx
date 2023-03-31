import React from 'react';

const LANGS = ['bun', 'deno', 'node', 'go', 'rust'];

function random(min: number, max: number) {
	const minimum = Math.ceil(min);

	return Math.floor(Math.random() * (Math.floor(max) - minimum + 1)) + minimum;
}

export default function HeroTerminal() {
	const isWindows =
		typeof window === 'undefined'
			? false
			: window.navigator.userAgent.toLowerCase().includes('win');
	const lang = LANGS[random(0, LANGS.length)] || LANGS[0];

	return (
		<ul
			className="flex flex-col w-full p-2 m-0 overflow-auto font-mono text-sm text-gray-200 border border-solid rounded-lg bg-slate-900 border-slate-500 list-none"
			style={{ height: 230 }}
		>
			<li className="text-gray-800"># Install proto</li>
			<li>
				{isWindows
					? 'irm https://moonrepo.dev/install/proto.ps1 | iex'
					: 'curl -fsSL https://moonrepo.dev/install/proto.sh | bash'}
			</li>

			{lang === 'bun' && (
				<React.Fragment key="bun">
					<li className="text-gray-800 pt-2"># Install Bun</li>
					<li>proto install bun 0.5</li>

					<li className="text-gray-800 pt-2"># Use immediately</li>
					<li>bun run index.ts</li>
				</React.Fragment>
			)}

			{lang === 'deno' && (
				<React.Fragment key="deno">
					<li className="text-gray-800 pt-2"># Install Deno</li>
					<li>proto install deno 1.31</li>

					<li className="text-gray-800 pt-2"># Use immediately</li>
					<li>deno run index.ts</li>
				</React.Fragment>
			)}

			{lang === 'node' && (
				<React.Fragment key="node">
					<li className="text-gray-800 pt-2"># Install Node.js</li>
					<li>proto install node 18</li>
					<li>proto install pnpm</li>

					<li className="text-gray-800 pt-2"># Use immediately</li>
					<li>pnpm install</li>
					<li>pnpm run dev</li>
				</React.Fragment>
			)}

			{lang === 'go' && (
				<React.Fragment key="go">
					<li className="text-gray-800 pt-2"># Install Go</li>
					<li>proto install go 1.20</li>

					<li className="text-gray-800 pt-2"># Use immediately</li>
					<li>go run .</li>
				</React.Fragment>
			)}

			{lang === 'rust' && (
				<React.Fragment key="rust">
					<li className="text-gray-800 pt-2"># Install Rust (requires rustup)</li>
					<li>proto install rust 1.68</li>

					<li className="text-gray-800 pt-2"># Use immediately</li>
					<li>cargo build</li>
				</React.Fragment>
			)}
		</ul>
	);
}
