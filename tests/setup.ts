// @ts-nocheck
import { plugin } from "bun";
import { compileModule } from "svelte/compiler";

plugin({
    name: "svelte-module-loader",
    setup(build) {
        build.onLoad({ filter: /\.svelte\.ts$/ }, async (args) => {
            const raw = await Bun.file(args.path).text();
            const transpiled = new Bun.Transpiler({ loader: "ts" }).transformSync(raw);
            const res = compileModule(transpiled, { filename: args.path });
            return {
                contents: res.js.code,
                loader: "js",
            };
        });
    },
});
