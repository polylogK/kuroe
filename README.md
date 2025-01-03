# kuroe

`kuroe` は，軽量かつステートレスな競技プログラミングの作問支援ツールです。

## 機能

- ジェネレータコードを用いたテストケースの生成（generate）
- 検証器コードを用いたテストケースの検証（validate）
- 想定解コードを用いたテストケースの解答生成（solve）
- ジャッジ（judge）

## サブコマンド：generate

ジェネレータコードからテストケースを生成します。

生成ケース数は，オプションまたはファイル名で指定可能です。
ファイル名による指定は，`kuroe.5.cpp` のようにジェネレータの拡張子の直前に指定します（この場合 5 個）。

```bash
$ kuroe generate example/generator/
[Generate] ████████████████████    5/5
+--------+-------------------------------------------+------------------------------------------+
| status | generated_case                            | from                                     |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_000.in" | "example/generator/example_by_cpp.3.cpp" |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_001.in" | "example/generator/example_by_cpp.3.cpp" |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_002.in" | "example/generator/example_by_cpp.3.cpp" |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_py_000.in"  | "example/generator/example_by_py.py"     |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_txt_000.in" | "example/generator/example_by_txt.in"    |
+--------+-------------------------------------------+------------------------------------------+
```

- 引数
  - `generators`：ジェネレータを含むディレクトリ or ジェネレータへのパス（複数可能）
- オプション
  - `-r`, `--recursive`：再帰的にジェネレータを探索するかどうか。
  - `-o`, `--outdir`：出力先ディレクトリ。デフォルトは `./testcases/input`
  - `-n`, `--count`：ファイルごとに n 個生成する。デフォルトは 1。ただしファイル名で指定されている場合はファイル名が優先。
  - `-s`, `--seed`：seed, seed+1, ..., seed+(n-1)。デフォルトは 0
  - `--tl`, `--timelimit`：生成のタイムリミット（秒）。デフォルトは 10.0
  - `-l`, `--language`：カスタム言語
- 出力
  - `outdir` に入力が生成される

### 補足

ジェネレータの実行では `./a.out 0` のように `seed` が渡されます。

## サブコマンド：validate

テストケースを検証します。
`testlib.h` による検証器を想定しています。

```bash
$ kuroe validate example/validator/
["example/validator/validator.cpp"] ████████████████████    5/5
+--------+-------------------------------------------+---------------------------------------------------------+
| status | target                                    | stderr                                                  |
+--------+-------------------------------------------+---------------------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_000.in" | "./testcases/validate/validator/example_by_cpp_000.val" |
+--------+-------------------------------------------+---------------------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_001.in" | "./testcases/validate/validator/example_by_cpp_001.val" |
+--------+-------------------------------------------+---------------------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_002.in" | "./testcases/validate/validator/example_by_cpp_002.val" |
+--------+-------------------------------------------+---------------------------------------------------------+
| OK     | "./testcases/input/example_by_py_000.in"  | "./testcases/validate/validator/example_by_py_000.val"  |
+--------+-------------------------------------------+---------------------------------------------------------+
| OK     | "./testcases/input/example_by_txt_000.in" | "./testcases/validate/validator/example_by_txt_000.val" |
+--------+-------------------------------------------+---------------------------------------------------------+
```

- 引数
  - `validator`：検証器を含むディレクトリ or 検証器へのパス（複数可能）
- オプション
  - `-r`, `--recursive`：再帰的に検証器を探索するかどうか
  - `-t`, `--testcases`：テストケースを含むディレクトリ or テストケースへのパス（複数可能）。デフォルトは `./testcases/input`
  - `-o`, `--outdir`：エラー出力先ディレクトリ。デフォルトは `./testcases/validate`
  - `-q`, `--quiet`：エラー出力を保存しない。
  - `-l`, `--language`：カスタム言語
- 出力
  - `--quiet` が指定されていない場合，`outdir` にエラー出力が生成される。

## サブコマンド：solve

想定解コードによって解答を生成します。

```bash
$ kuroe solve example/solver/correct.cpp
[Solve] ████████████████████    5/5
+--------+-------------------------------------------+---------------------------------------------+
| status | input                                     | generated_answer                            |
+--------+-------------------------------------------+---------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_000.in" | "./testcases/answer/example_by_cpp_000.ans" |
+--------+-------------------------------------------+---------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_001.in" | "./testcases/answer/example_by_cpp_001.ans" |
+--------+-------------------------------------------+---------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_002.in" | "./testcases/answer/example_by_cpp_002.ans" |
+--------+-------------------------------------------+---------------------------------------------+
| OK     | "./testcases/input/example_by_py_000.in"  | "./testcases/answer/example_by_py_000.ans"  |
+--------+-------------------------------------------+---------------------------------------------+
| OK     | "./testcases/input/example_by_txt_000.in" | "./testcases/answer/example_by_txt_000.ans" |
+--------+-------------------------------------------+---------------------------------------------+
```

- 引数
  - `solver`：想定解へのパス（1 つ）
- オプション
  - `-t`, `--testcases`：テストケースを含むディレクトリ or テストケースへのパス（複数可能）。デフォルトは `./testcases/input`
  - `-r`, `--recursive`：再帰的にテストケースを探索するかどうか
  - `-o`, `-outdir`：出力先ディレクトリ。デフォルトは `./testcases/answer`
  - `--tl`, `--timelimit`：生成のタイムリミット（秒）。デフォルトは 10.0
  - `-l`, `--language`：カスタム言語
- 出力
  - `outdir` に解答が生成される

## サブコマンド：judge

コードをジャッジします。

`.in` と `.ans` ファイルが揃っているテストケースを valid なケースと見なします。
`testlib.h` と同実行形式のチェッカーを使用することができます。

```bash
$ kuroe judge example/solver # 厳密一致によるジャッジ
[SOLVE "example/solver/correct.cpp"] ████████████████████    5/5
[JUDGE "example/solver/correct.cpp"] ████████████████████    5/5
+--------+---------------------------------------------+-----------------+
| status | input_and_answer                            | info            |
+--------+---------------------------------------------+-----------------+
| AC     | "./testcases/input/example_by_cpp_000.in"   | time = 9.3351ms |
|        | "./testcases/answer/example_by_cpp_000.ans" |                 |
+--------+---------------------------------------------+-----------------+
| AC     | "./testcases/input/example_by_cpp_001.in"   | time = 9.2928ms |
|        | "./testcases/answer/example_by_cpp_001.ans" |                 |
+--------+---------------------------------------------+-----------------+
| AC     | "./testcases/input/example_by_cpp_002.in"   | time = 7.6172ms |
|        | "./testcases/answer/example_by_cpp_002.ans" |                 |
+--------+---------------------------------------------+-----------------+
| AC     | "./testcases/input/example_by_py_000.in"    | time = 9.5468ms |
|        | "./testcases/answer/example_by_py_000.ans"  |                 |
+--------+---------------------------------------------+-----------------+
| AC     | "./testcases/input/example_by_txt_000.in"   | time = 9.0058ms |
|        | "./testcases/answer/example_by_txt_000.ans" |                 |
+--------+---------------------------------------------+-----------------+

[SOLVE "example/solver/wrong.cpp"] ████████████████████    5/5
[JUDGE "example/solver/wrong.cpp"] ████████████████████    5/5
+--------+---------------------------------------------+---------------------------------------------------+
| status | input_and_answer                            | info                                              |
+--------+---------------------------------------------+---------------------------------------------------+
| WA     | "./testcases/input/example_by_cpp_000.in"   | "./testcases/output/wrong/example_by_cpp_000.out" |
|        | "./testcases/answer/example_by_cpp_000.ans" |                                                   |
+--------+---------------------------------------------+---------------------------------------------------+
| WA     | "./testcases/input/example_by_cpp_001.in"   | "./testcases/output/wrong/example_by_cpp_001.out" |
|        | "./testcases/answer/example_by_cpp_001.ans" |                                                   |
+--------+---------------------------------------------+---------------------------------------------------+
| WA     | "./testcases/input/example_by_cpp_002.in"   | "./testcases/output/wrong/example_by_cpp_002.out" |
|        | "./testcases/answer/example_by_cpp_002.ans" |                                                   |
+--------+---------------------------------------------+---------------------------------------------------+
| WA     | "./testcases/input/example_by_py_000.in"    | "./testcases/output/wrong/example_by_py_000.out"  |
|        | "./testcases/answer/example_by_py_000.ans"  |                                                   |
+--------+---------------------------------------------+---------------------------------------------------+
| WA     | "./testcases/input/example_by_txt_000.in"   | "./testcases/output/wrong/example_by_txt_000.out" |
|        | "./testcases/answer/example_by_txt_000.ans" |                                                   |
+--------+---------------------------------------------+---------------------------------------------------+
$ kuroe judge example/solver -c example/checker.cpp # checker によるジャッジ
```

- 引数
  - `solver`：コードを含むディレクトリ or コードへのパス（複数可能）
- オプション
  - `-r`, `--recursive`：再帰的にコードを探索するかどうか
  - `-c`, `--checker`：チェッカーへのパス
  - `-t`, `--testcase`：テストケースを含むディレクトリ or テストケースへのパス（複数可能）。`.in` と `.ans` が揃っているケースのみジャッジ。再帰的に探索される。デフォルトは `./testcases`
  - `-o`, `--outdir`：ソルバ出力先ディレクトリ。デフォルトは `./testcases/output`
  - `--tl`, `--timelimit`：生成のタイムリミット（秒）。デフォルトは 2.0
  - `-l`, `--language`：カスタム言語
- 出力
  - `outdir` にソルバの出力が生成される

### カスタム言語

`kuroe` は，デフォルトで `C(gcc)`, `C++(g++)`, `Python`, `Txt(.in or .txt)` に対応しています。
この他の言語を使用したい場合や，コンパイラやオプション等を変更したい場合には，カスタム言語を使用することが可能です。

#### 例：g++ のオプションを変更する

`-l`, `--language` オプションによってカスタム言語を指定できます。
オプションには，「対象拡張子」「コンパイルコマンド（0 行以上）」「実行コマンド（1 行）」の三種類を `,` で区切り指定します。

```bash
kuroe judge idiot.cpp -l "(cpp|cc)","g++ -O3 -std=c++20 %(target)","./a.out"
```

## リファレンス兼謝辞

`kuroe` 実装にあたり以下を参考にしました。

- <https://rime.readthedocs.io/ja/latest/>
- <https://github.com/terry-u16/pahcer>
- <https://github.com/MikeMirzayanov/testlib/tree/master>
- <https://github.com/riantkb/testlib_for_yukicoder>

## kuroe?

`creating KyoUpRO problEm tool`
