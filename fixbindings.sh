#!/bin/bash

# This just deletes (or comments out) the #[link_name = "xxx"] name mangling 
# attributes in the bindings source file created by bindgen.
#
# This should just be a short-term fix until the issue can be resolved

# This would comment out the linker mangling
#sed -i 's/#\[link_name/\/\/#\[link_name/' $(find -name bindings.rs)

# This just deletes it
sed -i '/#\[link_name/d' $(find -name bindings.rs)

