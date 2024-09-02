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

### Install
