<h1>
  <img src="rustuxdows.svg" height="80" align="left" />
  wsl-tools
  <br />
  <sup><sub>
    <a href="README.md">English</a>
    &nbsp;│&nbsp;
    <a href="#インストール">インストール</a>
    &nbsp;│&nbsp;
    <a href="#xsel"><code>xsel</code></a>
    &nbsp;│&nbsp;
    <a href="#recycle"><code>recycle</code></a>
    &nbsp;│&nbsp;
    <a href="#open"><code>open</code></a>
  </sub></sup>
</h1>

WSL用のRust製クリップボード（xsel）とゴミ箱（recycle）とオーペン（open）のコマンドです。遅くて時々壊れる（Unicodeに正しく対応しないなど）PowerShellベースのソリューションにうんざりしたため、自作しました。私の作品のすべてと同じく完全に人間の手で書かれたものです。

それぞれのプログラムはLinuxバイナリもWindowsバイナリも含まれています。前者はパスの変換やパイプの確認を行った上で、WINAPIを呼び出すためにexeに処理を渡します。そのexeはWSL専用ではなく、バッチスクリプトなどで単独で使用することも可能です。

ボーナスとして、私はエクスプローラーのコンテキストメニューの「<img src="https://code.visualstudio.com/assets/apple-touch-icon.png" height="16" align="center" /> **Codeで開く**」をいつもWSLで開くようにするオプションをインストーラに追加しました。[ここで見える](https://stackoverflow.com/questions/59336461/how-to-open-vscode-remote-wsl-by-right-click)コンソールのチラつきや引用符の扱いの問題を避けるために、[もう一つの小さなバイナリ](src/bin/code-wsl.rs)でパスを変換してCodeを実行しています。

## インストール

> [!IMPORTANT]
> Linuxファイルシステム上のexeは、Windows上のexeよりも実行速度が10倍遅くなる場合があります。特にWindows 11でこの問題が顕著だと感じました。面倒を減らすために、バイナリをProgram FilesにコピーしてPATHに追加するインストーラを作成しました。

### [インストーラをダウンロード](https://github.com/maxkagamine/wsl-tools/releases/latest/download/wsl-tools-installer.exe)（おすすめ）

または[ZIPをダウンロード](https://github.com/maxkagamine/wsl-tools/releases/latest/download/wsl-tools-portable.zip)、またはソースからコンパイル：

1. [Rustをインストールして](https://rustup.rs/)（Windowsではなく、WSLで）
2. [Inno Setup 6.4+をインストールして](https://jrsoftware.org/isdl.php)（任意）
3. クロスコンパイルのために準備して：
   - Ubuntu: `sudo apt-get install mingw-w64 && rustup target add x86_64-pc-windows-gnu`
   - Arch: `sudo pacman -Syu mingw-w64 && rustup target add x86_64-pc-windows-gnu`
4. `make`を実行して

## xsel

Linuxでコピーしたり貼り付けたりするために使われる一般的なxselプログラムのドロップイン置換品です。多くのプログラムやクリップボードのライブラリがxselを探すため、これをPATHに入れるとWSLを意識していないソフトでもWindowsのクリップボードにコピーできるようになります。

```
使い方: xsel [オプション]

デフォルトでは、標準入力と標準出力の両方がターミナル（tty）であれば、クリップボードの内容が出
力されます。それ以外の場合は、標準出力がターミナル（tty）ではないとクリップボードの内容が出力
されて、標準入力がターミナル（tty）ではないとクリップボードが標準入力から読み込まれます。入力
オプションか出力オプションを使うと、プログラムが要求されたモードでのみ動作します。

入力と出力が両方必要な場合は、標準入力の内容に置き換えられる前に、前のクリップボードの内容が出
力されます。

入力オプション
  -a, --append            標準入力をクリップボードに追加する
  -f, --follow            ＜サポートされない＞
  -z, --zeroflush         ＜サポートされない＞
  -i, --input             標準入力をクリップボードに読み込む

出力オプション
  -o, --output            クリップボードを標準出力に書き出す

  --keep-crlf             ＜Windowsのみの追加＞ デフォルトでは、貼り付け時にCRLFがLFに
                          置き換える。このオプションを指定すると無効にできる。

操作オプション
  -c, --clear             クリップボードをクリアする
  -d, --delete            ＜サポートされない＞

セレクションオプション
  -p, --primary           PRIMARYとSECONDARYセレクションはWindowsに相当がないけど、
  -s, --secondary         いくつかのLinuxクリップボードマネージャーがセレクションと
  -b, --clipboard         クリップボードのバッファを同期させるから、このxselはそうだと
                          ふりして、選んだセレクションを無視する

  -k, --keep              ＜何もせずに終了＞
  -x, --exchange          ＜何もせずに終了＞

Xオプション
  --display               ＜サポートされない＞
  -m, --name              ＜サポートされない＞
  -t, --selectionTimeout  ＜サポートされない＞

その他のオプション
  --trim                  入力・出力の終わりから改行を消す
  -l, --logfile           ＜サポートされない＞
  -n, --nodetach          ＜無視される＞
  -h, --help              このヘルプを表示して終了
  -v, --verbose           ＜無視される＞
  --version               バーション情報を表示して終了
```

## recycle

[ソースで注釈](src/recycle_bin.rs)を参照してください。

Windowsのごみ箱が英語で「Recycle Bin」と呼ばれているため、この名前を付けました。

> [!NOTE]
> WSLファイルシステム上のファイルをごみ箱に移動してみる時に「Element not found.」というエラーが出る場合は、`wsl.exe --update`を実行、および/または再起動してみてください。これは\\\\wsl.localhostパスに関するWindowsのバグのようですが、原因は不明です（[microsoft/WSL#12444](https://github.com/microsoft/WSL/issues/12444), [microsoft/WSL#11252](https://github.com/microsoft/WSL/issues/11252)）。

```
使い方: recycle [オプション] <パス>...

指定したファイルとディレクトリをごみ箱に移動する。

デフォルトの動作（--rmなし）は、ユーザーがエクスプローラーでファイルを削除した場合と同じ
ように、シェルの普通の進捗や確認ダイアログを表示して、エクスプローラーの元に戻す履歴に追加
することです。これはWindows APIの制限による：ダイアログなしでファイルをごみ箱に移動する
ことは、シェルが永久に削除してしまうリスクが伴うので不可能です。ゆえに、スクリプトでは
ユーザーが期待していない時にこのコマンドを--rmなしで使用してはならない。

引数:
  <パス>...
          カレントディレクトリからの相対、ごみ箱に移動するファイルやディレクトリ。
          Linuxパスは自動的にWindowsパスに変換される。

オプション:
  -f, --force
          存在しないファイルを無視して

      --rm
          すべてのダイアログを非表示にして、シェルがごみ箱に移動できないものを永久に
          削除させる。警告：

          • ごみ箱に移動できたはずのファイルが削除される可能性がある。
            詳細はrecycle_bin.rsのコメントを参照してください。

          • ディレクトリは再帰的に削除される。

          • WSLファイルシステム上でsudoが必要なファイルは、recycleを使って削除できない
          　（エクスプローラーには不可能だ）。

  -v, --verbose
          削除進捗をターミナルで表示する

  -h, --help
          ヘルプを表示する

  -V, --version
          バーション情報を表示する
```

## open

[ShellExecuteExW](https://learn.microsoft.com/ja-jp/windows/win32/api/shellapi/nf-shellapi-shellexecuteexw)の簡単なラッパーです。

> [!IMPORTANT]
> .bashrcとかに`export BROWSER='open'`に追加してください。
> 
> UbuntuとDebianでは[alternativesシステム](https://manpages.debian.org/wheezy/dpkg/update-alternatives.8.ja.html)が/usr/bin/openやデフォルトのブラウザでリンクを開くために使われる2つのコマンドのシンボリックリンクを管理しているかもしれません。そのリンクが代わりにwsl-toolsに指すようにできます：
> ```bash
> for cmd in open www-browser x-www-browser; do
>   sudo update-alternatives --install "/usr/bin/$cmd" "$cmd" '/mnt/c/Program Files/wsl-tools/open' 999
> done
> ```

> [!TIP]
> `open -e image.png`は、image.pngを右クリックして「編集」を選択するのと同じです。デフォルトではペイントで開くんですが、Winaero Tweakerを使うか、regeditでこのキーを変更することで、好みの画像編集ソフトに設定できます： `HKEY_CLASSES_ROOT\SystemFileAssociations\image\shell\edit\command`

```
使い方: open [オプション] <パス>...

指定されたファイルまたはURLをデフォルトのプログラムで開きます（ディレクトリは
エクスプローラーで開きます）。

--verbに関する詳細は、次を参照してください：
https://learn.microsoft.com/ja-jp/windows/win32/shell/launch#object-verbs

引数:
  <パス>...  開くファイル、ディレクトリ、および/またはURL。Linuxパスは自動的に
             Windowsパスに変換される。

オプション:
  -e, --edit         --verb editのエイリアス
      --runas        --verb runasのエイリアス
      --verb <VERB>  実行すべき動詞
  -h, --help         ヘルプを表示する
  -V, --version      バーション情報を表示する
```

## 法的事項

Copyright © 鏡音マックス  
[Apache License 2.0](LICENSE.txt)の下でライセンスされています

## 違法事項

[海賊！](https://www.youtube.com/watch?v=NSZhIAfR6dA)
