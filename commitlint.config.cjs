/** @type {import('@commitlint/types').UserConfig} */
module.exports = {
  extends: ["@commitlint/config-conventional"],
  rules: {
    "scope-enum": [
      1,
      "always",
      [
        "api",
        "web",
        "workers",
        "db",
        "ci",
        "dx",
        "docs",
        "docker",
        "config",
        "observability",
        "auth",
        "platform",
        "shared",
        "deps",
        "release",
      ],
    ],
    "subject-case": [2, "never", ["start-case", "pascal-case", "upper-case"]],
    "header-max-length": [2, "always", 100],
  },
};
