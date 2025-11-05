# Anna Assistant - Feature Roadmap

This document tracks requested features and improvements from user feedback.

## 🔥 CRITICAL (Blocks Basic Functionality)

### 1. Show Command Output When Applying
**Status:** ⏳ IN PROGRESS

**Issue:** When applying recommendations, users can't see what's happening.
- Package installations appear frozen
- No progress indication
- Users think Anna is dead/hanging

**Solution:**
- [ ] Show command output in real-time in TUI (modal overlay)
- [ ] Stream stdout/stderr as command executes  
- [ ] Show progress for long-running operations
- [ ] Allow user to close output window when done

**User Quote:** *"I thought it was dead... we need an overlapping window terminal would be ideal)"*

---

## 🚨 HIGH PRIORITY (Major UX Issues)

### 2. Remove Applied Items from List Immediately
**Status:** 📋 TODO

**Issue:** Applied items disappear only after refresh, causing confusion.

**Solution:**
- [ ] Remove item from advice list immediately after successful apply
- [ ] Or show with strikethrough/greyed out
- [ ] Refresh only updates other items

### 3. Improve "Multiple System Errors" Details  
**Status:** ✅ PARTIALLY DONE

**What's Done:**
- ✅ Shows 3 actual error samples in the reason
- ✅ Explains advice lifecycle
- ✅ Uses less pager

**Still TODO:**
- [ ] Detect common error patterns
- [ ] Suggest specific fixes
- [ ] Link to wiki articles for error types

---

## 📊 MEDIUM PRIORITY (Smart Features)

### 4. Auto-Detect and Improve Current Terminal
**Status:** 📋 TODO

**Solution:**
- [ ] Detect current terminal emulator
- [ ] Suggest terminal-specific improvements
- [ ] Check emoji/icon support

### 5. Intelligent Config File Improvements
**Status:** 📋 TODO

**Examples:** vim syntax highlighting, shell aliases, terminal configs

**Solution:**
- [ ] Detect config files for installed software
- [ ] Suggest improvements from best practices
- [ ] Auto-apply common configs

---

## 🌟 LONG-TERM (Advanced Intelligence)

### 6. Full Arch Wiki Integration
**Status:** 💭 PLANNING

**Vision:** Anna reads Arch Wiki and applies intelligent solutions automatically.

**Solution:**
- [ ] Parse wiki pages for configs and solutions
- [ ] Build knowledge base
- [ ] Apply wiki-based recommendations
- [ ] Context-aware intelligent fixes

**User Quote:** *"anna knows best :)"*

---

**Last Updated:** 2025-11-05  
**User Feedback Session:** Beta.59
