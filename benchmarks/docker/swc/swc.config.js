module.exports = {
  // Core SWC configuration for benchmarking
  jsc: {
    target: "es2020",
    parser: {
      syntax: "typescript",
      tsx: true,
      decorators: true,
      dynamicImport: true
    },
    transform: {
      react: {
        runtime: "automatic"
      },
      optimizer: {
        globals: {
          vars: {
            "__DEBUG__": "false"
          }
        }
      },
      constModules: {
        globals: {
          "@swc/helpers": {
            asyncToGenerator: "asyncToGenerator",
            classCallCheck: "classCallCheck"
          }
        }
      }
    },
    minify: {
      compress: {
        arguments: false,
        arrows: true,
        booleans: true,
        booleans_as_integers: false,
        collapse_vars: true,
        comparisons: true,
        computed_props: true,
        conditionals: true,
        dead_code: true,
        directives: true,
        drop_console: false,
        drop_debugger: true,
        evaluate: true,
        expression: false,
        hoist_funs: false,
        hoist_props: true,
        hoist_vars: false,
        if_return: true,
        join_vars: true,
        keep_classnames: false,
        keep_fargs: true,
        keep_fnames: false,
        keep_infinity: false,
        loops: true,
        negate_iife: true,
        properties: true,
        reduce_funcs: false,
        reduce_vars: false,
        side_effects: true,
        switches: true,
        typeofs: true,
        unsafe: false,
        unsafe_arrows: false,
        unsafe_comps: false,
        unsafe_Function: false,
        unsafe_math: false,
        unsafe_symbols: false,
        unsafe_methods: false,
        unsafe_proto: false,
        unsafe_regexp: false,
        unsafe_undefined: false,
        unused: true
      },
      mangle: {
        props: {
          reserved: []
        }
      }
    }
  },
  module: {
    type: "es6",
    strict: false,
    strictMode: true,
    lazy: false,
    noInterop: false
  },
  sourceMaps: true,
  inlineSourcesContent: false
};