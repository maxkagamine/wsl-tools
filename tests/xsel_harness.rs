// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use assert_cmd::prelude::*;
use std::{
    io::Write,
    process::{Command, Stdio},
};

/// Test harness for running the xsel binary.
pub struct XselHarness<'a> {
    cmd: Command,
    stdin: Option<&'a str>,
    stdout_is_tty: bool,
}

impl<'a> XselHarness<'a> {
    /// Initializes a new xsel harness.
    ///
    /// # Panics
    /// Cargo failed to find the xsel binary.
    #[must_use]
    pub fn new() -> Self {
        let mut cmd = Command::cargo_bin("xsel").unwrap();
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        Self {
            cmd,
            stdin: None,
            stdout_is_tty: true,
        }
    }

    /// Adds arguments to the xsel command.
    pub fn args(&mut self, args: &[&str]) -> &mut Self {
        self.cmd.args(args);
        self
    }

    /// Sets the text to be fed into stdin when the command is run. This will cause stdin to appear
    /// as a pipe rather than a tty.
    pub fn stdin(&mut self, input: &'a str) -> &mut Self {
        self.stdin = Some(input);
        self
    }

    /// Sets whether stdout will appear to be a tty (default `true`) or redirected (`false`). The
    /// command output is captured either way.
    pub fn stdout_is_tty(&mut self, is_tty: bool) -> &mut Self {
        self.stdout_is_tty = is_tty;
        self
    }

    /// Runs xsel with the configured arguments and pipe setup and returns its stdout.
    ///
    /// # Panics
    /// - Failed to execute or wait for the command.
    /// - The command exited with a non-zero exit code.
    /// - Could not write to stdin.
    /// - Output could not be read as UTF-8.
    pub fn run(&mut self) -> String {
        // Leverage the same hidden flags that the Linux wrapper uses in order to simulate a tty
        self.cmd
            .arg(format!("--stdin-is-tty={}", self.stdin.is_none()))
            .arg(format!("--stdout-is-tty={}", self.stdout_is_tty));

        let mut child = self.cmd.spawn().unwrap();

        if let Some(text) = self.stdin {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(text.as_bytes())
                .unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{output:?}");

        String::from_utf8(output.stdout).unwrap()
    }
}

impl Default for XselHarness<'_> {
    fn default() -> Self {
        Self::new()
    }
}
