<p align="center">
	<img src="https://raw.githubusercontent.com/fossable/fossable/master/emblems/cci.svg" style="width:90%; height:auto;"/>
</p>

![License](https://img.shields.io/github/license/fossable/cci)
![Build](https://github.com/fossable/cci/actions/workflows/test.yml/badge.svg)
![GitHub repo size](https://img.shields.io/github/repo-size/fossable/cci)
![Stars](https://img.shields.io/github/stars/fossable/cci?style=social)

<hr>

**cci** (common-ci) is a tool that generates CI/CD configurations for popular
platforms like Github Actions and Gitlab CI. Imagine Terraform, but for CI
pipelines.

There are three main advantages to generating your CI workflows/pipelines:

- You can get started really quickly for projects in popular ecosystems.
- You're not locked into a single CI platform since you can easily generate
  pipelines for any platform.
- You don't have to write any Yaml because we have a TUI interface.

The downside is, of course, you don't get the full flexiblity of writing your
own pipeline from scratch. You can use a CCI pipeline as a starting point and
make your own customizations, but then you're not able to freely switch to
another CI platform.

## Supported CI platforms

- Github Actions
- Gitlab
- Circle CI
- Jenkins
- Gitea

## `common-ci.ron`

We use a configuration file called `common-ci.ron` in the root of your repo as
the _single source of truth_ definiton for your CI pipeline. Using this file,
`cci` can generate the proper CI configuration for any CI platform.

You can use our TUI interface to edit this file or any editor with LSP support.
Configure your editor to use `cci lsp` as an LSP and you'll get inline
documenation and autocomplete.

Here's an example of what a `common-ci.ron` file might look like:

```ron
(
  version: "1",
  presets: [
    Docker((
      registry: "hub.docker.com",
      image_name: "example",
      push_on_tags_only: true,
    )),
    Rust((
      enable_linter: true, // Clippy
    )),
  ]
)
```

# Presets

TODO list all presets behind a "click to show".
