import livereload from 'rollup-plugin-livereload'
import { nodeResolve } from "@rollup/plugin-node-resolve";
import rust from '@wasm-tool/rollup-plugin-rust'
import serve from 'rollup-plugin-serve'
import { terser } from 'rollup-plugin-terser'

const is_watch = !!process.env.ROLLUP_WATCH

export default {
  input: {
    index: './Cargo.toml'
  },
  output: {
    dir: 'dist/js',
    format: 'iife',
    sourcemap: true
  },
  plugins: [
    rust({
      serverPath: 'js/'
    }),

    nodeResolve({
      jail: __dirname
    }),

    is_watch &&
      serve({
        contentBase: 'dist',
        open: false
      }),

    is_watch && livereload('dist'),

    !is_watch && terser()
  ]
}
