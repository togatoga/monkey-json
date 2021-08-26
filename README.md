# monkey-json

`monkey-json` project is just for fun. I want to write JSON parser from the scratch.  
The implementation of `monkey-json` conforms to [RFC8259](https://www.rfc-editor.org/rfc/rfc8259) as possible as I can.  
I disregarded some rules on [RFC8259](https://www.rfc-editor.org/rfc/rfc8259). Because It's super boring to keep some of them(especially `Number`).  

You can use `monkey-json` as a command line tool(`mj`).

## mj

`mj` is a command line JSON minimum prettier like [`jq` ](https://github.com/stedolan/jq).

![Alt Text](https://raw.githubusercontent.com/togatoga/monkey-json/main/demo.gif)


### install
```bash
cargo install --git https://github.com/togatoga/monkey-json
```

### How to

```bash
% mj --help                                                       
mj - command line JSON minimum prettier
USAGE:
      mj [OPTIONS...] [FILE] [OPTIONS...]
ARGS:
     <FILE> A JSON file
OPTIONS:
       -h,--help      Print help information
       -c,--color     Color JSON output
       -m,--minimize  Minimize JSON output
# basic
% echo '{"key": "value"}' | mj   
{
   "key": "value"
}
# `-c` or `--color`
% mj --color example.json
# `-m` or `--minimize`
% mj --minimize example.json
```
