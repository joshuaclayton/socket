# Socket

Alpha software. Templating language akin to [Haml](http://haml.info/).

## page.skt

```
!HTML(lang=en)
%head
  %meta(charset=utf-8)
  %meta(http-equiv=x-ua-compatible content="ie=edge")
%body
  %section.primary
    %h2 What are you even doing?
    %ul
      %li This is an item
      %li This is another item
      %li.final This is the last item
```

## Generate HTML

```sh
cat page.skt | socket > index.html
```

## index.html

```
<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"></meta><meta http-equiv="x-ua-compatible" content="ie=edge"></meta></head><body><section class="primary"><h2>What are you even doing?</h2><ul><li>This is an item</li><li>This is another item</li><li class="final">This is the last item</li></ul></section></body></html>
```

## License

Copyright 2020 Josh Clayton. See the [LICENSE](LICENSE).
