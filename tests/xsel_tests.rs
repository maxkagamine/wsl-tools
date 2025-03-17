// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

mod xsel_harness;
use xsel_harness::XselHarness;

mod clipboard_via_powershell;
use clipboard_via_powershell::*;

#[test]
fn pastes_if_both_stdin_and_stdout_are_ttys() {
    let expected = "\
世界で一番おひめさま
そういう扱い心得てよね
";

    set_clipboard_via_powershell(expected);

    let actual = XselHarness::new().run();

    assert_eq!(
        actual, expected,
        "should have output the contents of the clipboard"
    );
    assert_eq!(
        actual,
        get_clipboard_via_powershell(),
        "the clipboard should remain unchanged"
    );
}

#[test]
fn copies_without_pasting_if_stdin_is_redirected() {
    let expected = "\
その一　いつもと違う髪形に気が付くこと
その二　ちゃんと靴まで見ること　いいね？
";

    set_clipboard_via_powershell("this should not be output");

    let output = XselHarness::new().stdin(expected).run();
    let actual = get_clipboard_via_powershell();

    assert_eq!(actual, expected, "should have copied to the clipboard");
    assert_eq!(
        output, "",
        "should not have output the contents of the clipboard"
    );
}

#[test]
fn copies_and_pastes_if_both_stdin_and_stdout_are_redirected() {
    let previous_clipboard = "\
その三　わたしの一言には三つの言葉で返事すること
わかったら右手がお留守なのを　なんとかして!
";

    let expected = "\
別にわがままなんて言ってないんだから
キミに心から思って欲しいの　かわいいって
";

    set_clipboard_via_powershell(previous_clipboard);

    let output = XselHarness::new()
        .stdin(expected)
        .stdout_is_tty(false)
        .run();

    let actual = get_clipboard_via_powershell();

    assert_eq!(actual, expected, "should have copied to the clipboard");
    assert_eq!(
        output, previous_clipboard,
        "should have output the contents of the clipboard before copying the new text"
    );
}

#[test]
fn input_copies_even_if_stdout_is_redirected() {
    let expected = "\
世界で一番おひめさま
気が付いて　ねえねえ
";

    set_clipboard_via_powershell("this should not be output");

    let output = XselHarness::new()
        .args(&["-i"])
        .stdin(expected)
        .stdout_is_tty(false)
        .run();

    let actual = get_clipboard_via_powershell();

    assert_eq!(actual, expected, "should have copied to the clipboard");
    assert_eq!(
        output, "",
        "should not have output the contents of the clipboard"
    );
}

#[test]
fn output_pastes_even_if_stdin_is_redirected() {
    let expected = "\
待たせるなんて論外よ
わたしを誰だと思ってるの？
";

    set_clipboard_via_powershell(expected);

    let actual = XselHarness::new()
        .args(&["-o"])
        .stdin("stdin should be ignored")
        .run();

    assert_eq!(
        actual, expected,
        "should have output the contents of the clipboard"
    );
    assert_eq!(
        actual,
        get_clipboard_via_powershell(),
        "the clipboard should remain unchanged"
    );
}

#[test]
fn input_and_output_together_copies_and_pastes() {
    let previous_clipboard = "\
もう何だか　あまいものが食べたい！
いますぐによ
";

    let expected = "\
欠点？かわいいの間違いでしょ
文句は許しませんの
";

    set_clipboard_via_powershell(previous_clipboard);

    let output = XselHarness::new()
        .args(&["-io"])
        .stdin(expected)
        .stdout_is_tty(true)
        .run();

    let actual = get_clipboard_via_powershell();

    assert_eq!(actual, expected, "should have copied to the clipboard");
    assert_eq!(
        output, previous_clipboard,
        "should have output the contents of the clipboard before copying the new text"
    );
}

#[test]
fn appends_to_clipboard() {
    let previous_clipboard = "\
あのね？私の話ちゃんと聞いてる？ちょっとぉ・・・
あ、それとね？白いおうまさん　決まってるでしょ？
";

    let text_to_append = "\
迎えに来て
わかったらかしずいて　手を取って「おひめさま」って
";

    let expected = format!("{previous_clipboard}{text_to_append}");

    set_clipboard_via_powershell(previous_clipboard);

    let output = XselHarness::new().args(&["-a"]).stdin(text_to_append).run();
    let actual = get_clipboard_via_powershell();

    assert_eq!(actual, expected, "should have appended to the clipboard");
    assert_eq!(
        output, "",
        "should not have output the contents of the clipboard"
    );
}

#[test]
fn outputs_and_appends_to_clipboard() {
    // Since append reuses the previously-read clipboard contents if possible, this also tests that.
    let previous_clipboard = "べつに　わがままなんて言ってないんだから";
    let text_to_append = "でもね　少しくらい叱ってくれたっていいのよ？";
    let expected = format!("{previous_clipboard}{text_to_append}");

    set_clipboard_via_powershell(previous_clipboard);

    let output = XselHarness::new()
        .args(&["-ao"])
        .stdin(text_to_append)
        .run();
    let actual = get_clipboard_via_powershell();

    assert_eq!(actual, expected, "should have appended to the clipboard");
    assert_eq!(
        output, previous_clipboard,
        "should have output the contents of the clipboard before appending"
    );
}

#[test]
fn clears_the_clipboard() {
    set_clipboard_via_powershell("this should be cleared");

    let output = XselHarness::new()
        .args(&["-c"])
        .stdin("stdin should be ignored")
        .stdout_is_tty(false)
        .run();

    assert!(
        !clipboard_contains_text(),
        "clipboard should have been cleared"
    );
    assert_eq!(
        output, "",
        "stdout being redirected should have had no effect"
    );
}

#[test]
fn outputs_and_clears_the_clipboard() {
    let previous_clipboard = "\
べつに　わがままなんて言ってないんだから
でもね　少しくらい叱ってくれたっていいのよ？
";

    set_clipboard_via_powershell(previous_clipboard);

    let output = XselHarness::new().args(&["-co"]).run();

    assert_eq!(
        output, previous_clipboard,
        "should have output the contents of the clipboard before clearing it"
    );
    assert!(
        !clipboard_contains_text(),
        "clipboard should have been cleared"
    );
}

#[test]
fn outputs_nothing_without_error_if_clipboard_does_not_contain_text() {
    // Clipboard is not empty as in "" but rather does not contain a CF_TEXT or CF_UNICODETEXT.
    clear_clipboard_via_powershell();

    let output = XselHarness::new().args(&["-o"]).run();
    assert_eq!(output, "", "should have output nothing");
}

#[test]
fn crlf_is_replaced_with_lf_by_default() {
    let text_on_clipboard = "\
世界でわたしだけのおうじさま\r\n\
気が付いて　ほらほら\r\n\
おててが空いてます\r\n\
無口で無愛想なおうじさま\r\n\
もう　どうして！　気が付いてよ早く\r\n";

    let expected = text_on_clipboard.replace("\r\n", "\n");

    set_clipboard_via_powershell(text_on_clipboard);

    let actual = XselHarness::new().run();

    assert_eq!(
        actual, expected,
        "should have output the contents of the clipboard with CRLFs replaced with LF"
    );
}

#[test]
fn keep_crlf_preserves_line_endings() {
    let expected = "\
世界でわたしだけのおうじさま\r\n\
気が付いて　ほらほら\r\n\
おててが空いてます\r\n\
無口で無愛想なおうじさま\r\n\
もう　どうして！　気が付いてよ早く\r\n";

    set_clipboard_via_powershell(expected);

    let actual = XselHarness::new().args(&["--keep-crlf"]).run();

    assert_eq!(
        actual, expected,
        "should have output the contents of the clipboard with CRLFs kept as-is"
    );
}

#[test]
fn trims_newlines_from_input_and_output() {
    let previous_clipboard = "\
いちごの乗ったショートケーキ\r\n\
こだわりたまごのとろけるプリン\r\n\
みんな　みんな　我慢します…　　\r\n"; // Should only trim newlines, not spaces

    // Leading newlines should not be removed
    let new_clipboard = "

わがままな子だと思わないで
わたしだってやればできるもん
あとで後悔するわよ　　
";

    let expected_previous_clipboard = previous_clipboard
        .trim_end_matches("\r\n")
        .replace("\r\n", "\n");

    let expected_new_clipboard = new_clipboard.trim_end_matches('\n');

    set_clipboard_via_powershell(previous_clipboard);

    let actual_previous_clipboard = XselHarness::new()
        .args(&["-io", "--trim"])
        .stdin(new_clipboard)
        .run();

    let actual_new_clipboard = get_clipboard_via_powershell();

    assert_eq!(
        actual_new_clipboard, expected_new_clipboard,
        "should have copied to the clipboard with the trailing LF removed"
    );
    assert_eq!(
        actual_previous_clipboard, expected_previous_clipboard,
        "output should have CRLFs replaced with LF and the trailing newline removed"
    );
}

#[test]
fn trims_newlines_from_output_while_preserving_line_endings() {
    let previous_clipboard = "\
世界で一番おひめさま\r\n\
ちゃんと見ててよね　どこかに行っちゃうよ？\r\n\
ふいに抱きしめられた　急に　そんな　えっ？\r\n";

    let expected_previous_clipboard = previous_clipboard.trim_end_matches("\r\n");

    set_clipboard_via_powershell(previous_clipboard);

    let actual_previous_clipboard = XselHarness::new().args(&["--keep-crlf", "--trim"]).run();

    assert_eq!(
        actual_previous_clipboard, expected_previous_clipboard,
        "output should have trailing newline removed and the remaining CRLFs left as-is"
    );
}

#[test]
fn selection_options_are_ignored() {
    let expected = "\
「轢かれる　危ないよ」　そう言ってそっぽ向くキミ
…こっちのが危ないわよ
";

    clear_clipboard_via_powershell();

    XselHarness::new().args(&["-b"]).stdin(expected).run();
    let actual1 = XselHarness::new().args(&["-p"]).run();
    let actual2 = XselHarness::new().args(&["-s"]).run();

    assert_eq!(actual1, expected, "--primary and --clipboard are the same");
    assert_eq!(
        actual2, expected,
        "--secondary and --clipboard are the same"
    );
}

#[test]
fn keep_and_exchange_noop_and_exit() {
    let expected = "…こっちのが危ないわよ";

    set_clipboard_via_powershell(expected);

    let output1 = XselHarness::new()
        .args(&["-koi"])
        .stdin("stdin should be ignored")
        .run();

    let output2 = XselHarness::new()
        .args(&["-xoi"])
        .stdin("stdin should be ignored")
        .run();

    assert_eq!(output1, "", "should not have output anything");
    assert_eq!(output2, "", "should not have output anything");
    assert_eq!(
        get_clipboard_via_powershell(),
        expected,
        "should not have copied to clipboard"
    );
}
