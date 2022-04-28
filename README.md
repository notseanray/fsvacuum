<h3 align="center">
	<br>
	fsvacuum
	<br>
</h3>

<p align="center">multithreaded file cleaner</p>

<p align="center">
	<a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

fsvacuum is a multithreaded file cleaner to recursively remove js/ts `node_modules` folders, Rust `target` folders, and various cache folders.

##### why?

Due to the nature of cargo and npm, project folder size can build up over time making it harder to backup projects and other files. 

#### usage
print out all folders that will be affected: `fsvacuum js` (current options are `js`, `zig`, `cache`, `nvim` (nvim swap), or `rust`)

remove all affected folders: `fsvacuum clean rust` 

removing the cache folder will always delete the folders for go, pylint, typescript, yarn, chromium, pip, and mozilla (firefox). 

#### installation

it is recommended you have [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed

```
$ git clone git@github.com:NotCreative21/fsvacuum.git
$ cd fsvacuum
$ ./install.sh
```
