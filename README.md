# Pixiv Tagging for Hydrus

This program allows you to automatically tag files stored in hydrus with tags retrieved from 
pixiv by using saucenao.

## Installation

You need to have cargo installed and can just do 
```
cargo install hydrus-pixiv-tagger
```

Or build the binary yourself. You need a rust-toolchain installation (for example with [rustup](https://rustup.rs)).
```
git clone https://github.com/Trivernis/hydrus-pixiv-tagger.git
cd hydrus-pixiv-tagger
cargo build --release
```


## Usage

```
USAGE:
    hydrus-pixiv-tagger [FLAGS] [OPTIONS] --hydrus-key <hydrus-key> --saucenao-key <saucenao-key>

FLAGS:
    -h, --help       Prints help information
        --inbox      Searches in the inbox instead
    -V, --version    Prints version information

OPTIONS:
        --hydrus-key <hydrus-key>        The hydrus client api key
        --hydrus-url <hydrus-url>        The url to the hydrus client api [default: http://127.0.0.1:45869]
        --saucenao-key <saucenao-key>    The saucenao api key
        --tag-service <tag-service>      The tag service the tags will be assigned to [default: my tags]
    -t, --tags <tags>...                 Tags used to search for files

```

## Example

```
hydrus-pixiv-tagger 
    --hydus-key <key>\
    --hydrus-url http://127.0.0.1:45869 \
    --saucenao-key <key2>\
    --tag-service 'public tag repository'\
    --tags 'meta:tagme' 
```

```
hydrus-pixiv-tagger 
    --hydus-key <key>\
    --hydrus-url http://127.0.0.1:45869 \
    --saucenao-key <key2>\
    --inbox
    --tag-service 'my tags'
```

## License

Apache-2.0