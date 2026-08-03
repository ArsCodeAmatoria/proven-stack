/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: "no-feature-to-feature",
      comment: "Features must not import other features directly.",
      severity: "error",
      from: { path: "^features/([^/]+)" },
      to: { path: "^features/([^/]+)", pathNot: "^features/$1" },
    },
    {
      name: "no-ui-business-rules",
      comment: "UI components must not import hypothetical domain packages.",
      severity: "error",
      from: { path: "^components/" },
      to: { path: "(domain|@proven/domain)" },
    },
  ],
  options: {
    doNotFollow: {
      path: "node_modules",
    },
    tsPreCompilationDeps: true,
    baseDir: ".",
  },
};
