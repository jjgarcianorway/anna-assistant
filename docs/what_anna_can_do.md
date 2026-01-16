# What Anna Can Do, What She Cannot Do, and Why

This document explains Anna's abilities and limitations in plain language.
It is meant for users who want to understand what Anna can and cannot do
before they start using her.

## Who Is Anna?

Anna is a system assistant for Arch Linux. She can answer questions about
your computer, diagnose problems, and help fix common issues. She runs
locally on your machine - your questions and data never leave your computer.

## What Anna Can Do

### Read System Information

Anna can look at your computer's state to answer questions:

- **Hardware details**: What CPU, memory, disk space, and devices you have
- **WiFi status**: Connection status, signal strength, speed
- **System state**: Running services, loaded drivers, basic health
- **Configuration files**: Settings that affect how your system works

These are read-only operations. Anna looks at information but does not
change anything.

### Diagnose Problems

When something is wrong, Anna can investigate:

- **Slow WiFi**: Checks driver settings and recommends fixes
- **System errors**: Looks at logs and identifies issues
- **Hardware problems**: Detects missing drivers or misconfigured devices

Anna will show you what she found and explain what it means in plain language.

### Suggest Fixes

For problems Anna knows how to fix, she will:

1. **Explain the problem**: What is wrong and why it matters
2. **Show the solution**: The exact commands that would fix it
3. **Cite sources**: Links to documentation so you can verify the advice

Anna never runs fix commands herself. She shows you what to do and lets
you decide.

### Run Safe Diagnostic Commands (With Your Permission)

Some commands are safe for Anna to run because they only read information:

- `iw wlan0 link` - Check WiFi connection status
- `lsmod` - List loaded drivers
- `lspci` - Show hardware devices
- `free` - Show memory usage
- `df` - Show disk space
- `uname` - Show system version

Before running these, Anna asks for your explicit confirmation.
The confirmation is exact wording - no guessing or shortcuts.

## What Anna Cannot Do Automatically

### Commands That Need Your Action

Anna will never automatically run commands that:

- Require administrator (sudo) access
- Change system files
- Install or remove software
- Modify settings

For these operations, Anna shows you the commands and you copy them
into your terminal yourself. This keeps you in control.

### Example: WiFi Fix

When Anna diagnoses a WiFi problem, she might suggest:

```
sudo cp /etc/modprobe.d/iwlwifi.conf /etc/modprobe.d/iwlwifi.conf.backup
```

Anna will not run this command. She shows it to you, explains why it
helps, and lets you decide whether to run it.

## What Anna Will Never Do

These are permanent limits. Anna cannot do these things even if you ask.

### Never Download Files

Anna cannot fetch files from the internet. She has no access to
`wget`, `curl`, or any network tools. This prevents accidentally
downloading something harmful.

### Never Install Software

Anna cannot run package managers (`pacman`, `apt`, etc.). She can
tell you what to install, but you must do it yourself.

### Never Run Commands as Administrator

Anna cannot use `sudo`, `su`, or any tool that grants root access.
Administrator commands must always be run by you.

### Never Delete or Overwrite Files

Anna cannot run `rm`, `dd`, or any command that destroys data.
She can suggest cleanup, but the actual deletion is your choice.

### Never Access Other Computers

Anna cannot use `ssh`, `scp`, or any remote access tools.
She only works with your local machine.

## Why These Limits Exist

Anna's limits are not temporary - they are by design.

### Your Safety

A helpful assistant that can download files, install software, and
run admin commands is also an assistant that could be tricked into
doing harmful things. Anna's limits prevent this category of mistakes
entirely.

### Your Control

When you run a command yourself, you see exactly what happens.
You can stop if something looks wrong. You can undo it.
Anna's role is to help you understand, not to act for you.

### Transparency

Anna will tell you what she can and cannot do. If you ask her to
do something outside her limits, she will explain why she cannot
and suggest alternatives.

## How to See Anna's Capabilities

Run this command to see Anna's current capabilities:

```
annactl capabilities
```

This shows:
- What Anna can do
- What she cannot do automatically
- What she will never do
- The exact commands she is allowed to run

## Summary

| Category | What It Means |
|----------|---------------|
| **Can read** | Anna looks at system state (hardware, WiFi, logs) |
| **Can diagnose** | Anna identifies problems and explains them |
| **Can suggest** | Anna shows exact commands to fix issues |
| **Can run (with confirmation)** | Safe read-only commands like `lsmod`, `df` |
| **Cannot run automatically** | Commands requiring sudo or changing files |
| **Will never do** | Download, install, delete, or remote access |

Anna is designed to be helpful within clear boundaries.
She helps you understand and fix your system while keeping you in control.
