# Typst-live

`typst-preview` has the features listed below and likely has less bugs. 

https://github.com/Enter-tainer/typst-preview

This is a fork of typst-live. The original README is below. 

This fork uses multiple svg output files instead of a single pdf. This enables saving scrolling and zoom
information in browsers that do not support it (e.g. Arc, Chrome, Safari). Based off of some preliminary 
testing, Firefox and Edge already save this information. 

The typst compiler ouputs each page of a document into separate svg files and only outputs files when the 
specific file needs to be modified. Therefore, there is no need to reload the whole pdf, so this fork only
reloads changed svg files. This means there is no flicker, minimal delay, and (hopefully) less energy usage.

One additional change is that this fork uses a tmp folder instead of the working directory for the 
intermediate preview files.

Unfortunately, it appears that typst does not clean up pages if the page count reduces. That means
there will be dangling extra pages if the document gets shorter.

# Original README

This is a simple utility to watch for changes in your [typst](https://github.com/typst/typst) file and automatically
recompile them for live feedback. `typst-live` allows you to open a tab in your browser with typst generated pdf and have it automatically reload
whenever your source `.typ` files are changed.

## Difference from `--watch` flag
`typst-live` hosts a webserver that automatically refreshes the page so you don't have to manually reload it with `typst --watch`

## Installation
If you have [rust](https://www.rust-lang.org) setup use the following command:
```
cargo install typst-live
```

## Usage
### 1. With auto recompilation
* Launch `typst-live` from your terminal:
```
$ ./typst-live <file.typ>
Server is listening on http://127.0.0.1:5599/
```
* Go to `http://127.0.0.1:5599/` in your browser.
* Now edit your `file.typ` and watch changes appear in browser tab.

### 2. With manual recompilation
You can use `typst-live` to reload pdf files without recompilation of source files.
For that you want to use `--no-recompile` option which disables recompilation and just hosts
your pdf file in browser tab, you will need to specify `filename` as pdf instead of source `.typ` file.
Whenever pdf file changes browser tab will be refreshed.
