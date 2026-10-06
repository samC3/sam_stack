cat >> calc.py <<'PY'


def subtract(a, b):
    return a - b
PY
cat > test_calc.py <<'PY'
from calc import add, subtract

assert add(2, 3) == 5
assert subtract(5, 3) == 2
assert subtract(3, 5) == -2
print("ok")
PY
git add -A && git commit -qm "Add subtract"
