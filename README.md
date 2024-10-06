# nmt (node_modules trimmer)

nmt is a CLI utility that trims `node_modules` folder size by removing unnecessary files and folders.

## Why nmt?

When you're building a container or a lambda, the size of your application matters. A smaller artifact size means faster deployments, faster loading times and less storage used.

`node_modules` is one of the biggest contributors to the size of your application. By removing unnecessary files and folders, you can shrink the size of your application and make it more efficient.

## Features

* Remove unnecessary files and folders from `node_modules` (like: type definitions, tests, etc);
* Minify JavaScript files;
* Remove all CJS or ESM files.

## Usage

`nmt` provides 2 binaries: `cli` and `docker`.

### CLI

`cli`: is the binary that removes unnecessary files and folders from `node_modules`.

```bash
Usage: cli [OPTIONS]

Options:
  -p, --project-root-location <PROJECT_ROOT_LOCATION>
          Path to the project root
          
          [env: PROJECT_ROOT_LOCATION=]
          [default: .]

  -e, --entry-point-location <ENTRY_POINT_LOCATION>
          Path to the node_modules directory
          
          [env: ENTRY_POINT_LOCATION=]
          [default: dist/index.js]

  -H, --home-location <HOME_LOCATION>
          Path to the home directory
          
          [env: HOME_LOCATION=]
          [default: ~]

  -d, --dry-run
          Whether to perform a dry run
          
          [env: DRY_RUN=]

  -m, --minify
          Whether to minify JS files
          
          [env: MINIFY=]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Docker

`docker`: is the binary that builds a shrinked version of the desired Docker image.


```bash
Usage: docker [OPTIONS]

Options:
  -p, --project-root-location <PROJECT_ROOT_LOCATION>
          Path to the project root
          
          [env: PROJECT_ROOT_LOCATION=]
          [default: .]

  -e, --entry-point-location <ENTRY_POINT_LOCATION>
          Path to the node_modules directory
          
          [env: ENTRY_POINT_LOCATION=]
          [default: dist/index.js]

  -H, --home-location <HOME_LOCATION>
          Path to the home directory
          
          [env: HOME_LOCATION=]
          [default: ~]

  -d, --dry-run
          Whether to perform a dry run
          
          [env: DRY_RUN=]

  -m, --minify
          Whether to minify JS files
          
          [env: MINIFY=]

  -s, --source-image <SOURCE_IMAGE>
          The source image
          
          [env: SOURCE_IMAGE=]
          [default: hello-world]

  -D, --destination-image <DESTINATION_IMAGE>
          The destination image
          
          [env: DESTINATION_IMAGE=]
          [default: ]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Benchmarks

| image name         | size before | size after | commands                                                                                                                     |
| ------------------ | ----------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------- |
| nodered/node-red   | 569.82 MB   | 453.79 MB  | --source-image nodered/node-red --entry-point-location "node_modules/node-red/red.js" --keep "node_modules/oauth2orize/lib/**/*.*" --keep "**/*node-red/**/*.*" --keep "**/ajv/lib/refs/*.*"      |
| nodered/node-red   | 569.82 MB   | 447.7 MB  | --source-image nodered/node-red --entry-point-location "node_modules/node-red/red.js" --keep "node_modules/oauth2orize/lib/**/*.*" --keep "**/*node-red/**/*.*" --keep "**/ajv/lib/refs/*.*" --minify |
| ilteoood/xdcc-mule | 176.68 MB   | 141.91 MB  | --source-image ilteoood/xdcc-mule --entry-point-location "./index.js" --keep "**/node_sqlite3.node"
| ilteoood/xdcc-mule | 176.68 MB   | 139.3 MB  | --source-image ilteoood/xdcc-mule --entry-point-location "./index.js" --keep "**/node_sqlite3.node" --minify
| ghost | 595.73 MB | 402.22 MB | --source-image ghost --entry-point-location "index.js" --keep "**/node_sqlite3.node" --keep "**/bookshelf-relations/**/*.*" --keep "**/@tryghost/**/*.*" --keep "**/gscan/**/*.*" --keep "**/core/**/*.*"
| ghost | 595.73 MB | 378.48 MB | --source-image ghost --project-root-location /var/lib/ghost/current --entry-point-location "index.js" --keep "**/node_sqlite3.node" --keep "**/bookshelf-relations/**/*.*" --keep "**/@tryghost/**/*.*" --keep "**/gscan/**/*.*" --keep "**/core/**/*.*" --minify