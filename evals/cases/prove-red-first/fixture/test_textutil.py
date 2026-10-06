from textutil import slugify

assert slugify("Hello World") == "hello-world"
assert slugify("  Padded  ") == "padded"
print("ok")
