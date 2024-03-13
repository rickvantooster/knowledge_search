# Knowledge seach
A local document search engine with a tui.

## Features: 
1. Fast indexing via parallelization
2. Auto reindexing on file changes
3. Support for the following file types:
    * html
    * xhtml
    * docx
    * pdf
    * txt
4. Two different index file types:
    * Json:
        - Supports both stemmed and non stemmed terms
        - Exact phrase searching
    * Inverted-Index like:
        - Only supports stemmed terms
        - No exact phrase searching
        - Faster searching and Indexing

## Commandline usage:
```bash
$<Executable name> <Dir>
```

## Planned features
- Sqlite support
- Storing log file in home directory
- Additional file support
- Global config for parallelization core count and idex file format
- Compression for index files




