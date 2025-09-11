#!/usr/bin/env bash
set -euo pipefail
julia -e '
import Pkg
for p in ["HTTP","JSON","Statistics","Dates","Random"]
    try
        Pkg.add(p)
    catch e
        @warn("pkg add failed", p, e)
    end
end
'
echo "Julia deps ensured."
