# Kernel Cursor and Prompt Logic Documentation

## Overview

This document provides an exhaustive explanation of the cursor and prompt logic in the kernel, specifically focusing on how `keyboard::KeyEvents::Enter` integrates with the command handling system for both known and unknown commands. **This version documents critical cursor and prompt position mismatches that occur after command execution.**

## CRITICAL ISSUE: Cursor and Prompt Position Mismatches

The current implementation has several critical issues where the command handler's cursor position tracking becomes desynchronized with the actual screen cursor position after command execution. These issues cause the prompt to appear in wrong locations and cursor positioning to be incorrect.

### Problem Summary

1. **Command Handler vs Screen Cursor Desynchronization**: The command handler tracks its own cursor position but doesn't properly sync with the screen's actual cursor position after commands execute
2. **Prompt Position Calculation Errors**: The prompt position is calculated incorrectly after command output
3. **Missing Screen Updates**: After command execution, the screen cursor position is not properly updated
4. **Buffer Clearing Issues**: The command buffer is cleared but cursor positions are not reset properly

## Architecture Overview

The kernel implements a multi-screen terminal system with the following key components:

1. **Screen Manager**: Manages multiple virtual screens with a single physical display
2. **Command Handler**: Processes user input and executes commands
3. **Keyboard Driver**: Translates hardware scancodes into high-level events
4. **Main Event Loop**: Orchestrates all input/output operations

## Screen Management System

### Screen Manager Structure

```rust
pub struct ScreenManager {
    pub screens: [Option<Screen>; MAX_SCREENS],  // Virtual screens (max 3)
    pub active_screen_id: usize,                 // Currently active screen
    pub physical_buffer: &'static mut Buffer,    // Physical VGA buffer
}
```

### Screen Initialization

During kernel startup (`kernel_main()`):

1. **Screen Manager Initialization**: `init_screen_manager()` creates the global screen manager
2. **Screen Creation**: A second screen (ID 1) is created for the user terminal
3. **Welcome Message**: Screen 1 displays welcome message and initial prompt
4. **Screen Switching**: System starts on screen 0, user terminal is on screen 1

```rust
// From kernel_main()
if let Some(_screen_id) = manager.create_screen() {            
    if manager.switch_screen(1) {
        if let Some(screen) = &mut manager.screens[1] {
            let mut writer = Writer::new(screen);
            write!(writer, "#                             Welcome to the User Terminal                     #\n").unwrap();
            write!(writer, "\n").unwrap();
            write!(writer, "Type 'help' for available commands.\n").unwrap();
            write!(writer, "> ").unwrap();  // Initial prompt
        }
    }
    manager.switch_screen(0);  // Return to screen 0
}
```

## Command Handler System

### Command Handler Structure

```rust
pub struct CommandHandler {
    buffer: [u8; 256],        // Input buffer for command characters
    buffer_len: usize,        // Current length of command in buffer
    cursor_pos: usize,        // Cursor position within the buffer
    prompt_start_col: usize,  // Column where prompt starts
    prompt_start_row: usize,  // Row where prompt starts
}
```

**ISSUE**: The command handler now tracks cursor position internally, but this creates synchronization problems with the screen's actual cursor position.

### Command Processing Flow

The command handler maintains a character buffer that accumulates user input until Enter is pressed:

1. **Character Input**: Each typed character is added to the buffer via `add_char()`
2. **Command Execution**: When Enter is pressed, `execute_command()` processes the buffer
3. **Buffer Clearing**: After execution, the buffer is cleared for the next command

## Keyboard Event Processing

### Main Event Loop

The kernel runs a continuous event loop that polls for keyboard input:

```rust
loop {
    if let Some(key_event) = keyboard::poll_keyboard() {
        match key_event {
            // Character input
            keyboard::KeyEvents::Character(c) => {
                // Handle character input
            }
            // Enter key processing
            keyboard::KeyEvents::Enter => {
                // Handle command execution
            }
            // Other events...
        }
    }
}
```

### Character Input Handling

When a character is typed on screen 1 (the user terminal):

```rust
keyboard::KeyEvents::Character(c) => {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    
    if active_screen_id == 1 {
        drop(manager);
        
        let mut cmd_handler = command_handler().lock();
        cmd_handler.add_char(c as u8);
    } else {
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            writer.write_byte(c as u8);
        }
        manager.flush_to_physical();
        manager.update_cursor();
    }
}
```

**ISSUE**: On screen 1, characters are only added to the command buffer but NOT displayed on screen. The command handler's `add_char()` method handles the display internally, but this creates a disconnect between the main loop and the screen updates.

**Key Points:**
- Characters are only added to the command buffer on screen 1
- Screen display is handled by the command handler, not the main loop
- This creates synchronization issues between the command handler and screen manager

## Enter Key Processing Logic

### The Enter Key Event Handler

The `keyboard::KeyEvents::Enter` handler implements different behavior based on the active screen:

```rust
keyboard::KeyEvents::Enter => {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    
    if active_screen_id == 1 {
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            writer.write_byte(b'\n');
        }
        manager.flush_to_physical();
        manager.update_cursor();
        
        // Release manager lock before executing command
        drop(manager);
        
        // Execute the command
        {
            let mut cmd_handler = command_handler().lock();
            cmd_handler.execute_command();
        }
        
        // Show prompt again and set prompt position
        {
            let mut manager = screen_manager().lock();
            if let Some(screen) = &mut manager.screens[1] {
                let mut writer = Writer::new(screen);
                write!(writer, "> ").unwrap();
                
                // Get current cursor position for prompt - using correct field names
                let prompt_row = screen.row_position;
                let prompt_col = screen.column_position;
                
                // Update prompt position in command handler
                drop(manager);
                let mut cmd_handler = command_handler().lock();
                cmd_handler.set_prompt_position(prompt_row, prompt_col);
            }
            else {
                manager.flush_to_physical();
                manager.update_cursor();
            }
        }
    } else {
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            writer.write_byte(b'\n');
        }
        manager.flush_to_physical();
        manager.update_cursor();
    }
}
```

**CRITICAL ISSUES IN THIS CODE:**

1. **Missing Screen Update After Command Execution**: After `cmd_handler.execute_command()`, the screen is not flushed or cursor updated
2. **Prompt Position Race Condition**: The prompt position is set AFTER writing the prompt, but the command handler needs it BEFORE
3. **Lock Management Issues**: Multiple lock acquisitions and releases create potential race conditions
4. **Incomplete Error Handling**: If screen access fails, the cursor is not updated

### Step-by-Step Enter Processing

1. **Screen Check**: Determine if we're on screen 1 (user terminal)
2. **Newline Display**: Add a newline character to move to next line
3. **Screen Update**: Flush changes to physical display and update cursor
4. **Lock Management**: Release screen manager lock to prevent deadlocks
5. **Command Execution**: Process the command in the buffer
6. **Prompt Restoration**: Display the "> " prompt for the next command
7. **Prompt Position Update**: Set the command handler's prompt position
8. **Final Update**: Flush and update cursor position

**ISSUE**: Step 7 happens AFTER step 6, but the command handler needs the prompt position BEFORE it can properly handle character input.

## Command Execution Flow

### Command Handler Execution

The `execute_command()` method processes the accumulated input:

```rust
pub fn execute_command(&mut self) -> bool {
    if self.buffer_len == 0 {
        return false;  // Empty command, do nothing
    }

    // Convert buffer to string for parsing
    let command_str = core::str::from_utf8(&self.buffer[..self.buffer_len])
        .unwrap_or("");

    let command = self.parse_command(command_str);
    self.handle_command(command);
    
    // Clear the buffer after execution
    self.clear_buffer();
    true
}
```

**ISSUE**: The `clear_buffer()` method resets `buffer_len` and `cursor_pos` but doesn't update the screen cursor position or sync with the screen manager.

### Command Parsing

Commands are parsed using a simple string matching approach:

```rust
fn parse_command(&self, input: &str) -> Command {
    match input.trim() {
        "reboot" => Command::Reboot,
        "clear" => Command::Clear,
        "help" => Command::Help,
        _ => Command::Unknown,  // Any unrecognized command
    }
}
```

### Command Handling

Each command type has a dedicated execution method:

```rust
fn handle_command(&self, command: Command) {
    match command {
        Command::Reboot => self.execute_reboot(),
        Command::Clear => self.execute_clear(),
        Command::Help => self.execute_help(),
        Command::Unknown => self.execute_unknown(),  // Handles unknown commands
    }
}
```

## Known vs Unknown Command Processing

### Known Commands

Known commands (`help`, `clear`, `reboot`) have specific implementations:

- **Help**: Displays available commands
- **Clear**: Clears the screen and resets cursor
- **Reboot**: Performs system restart

### Unknown Command Processing

Unknown commands are handled by `execute_unknown()`:

```rust
fn execute_unknown(&self) {
    let mut manager = screen_manager().lock();
    if let Some(screen) = &mut manager.screens[1] {
        let mut writer = Writer::new(screen);
        write!(writer, "Unknown command. Type 'help' for available commands.\n").unwrap();
    }
    manager.flush_to_physical();
}
```

**CRITICAL ISSUE**: This method flushes to physical but does NOT update the cursor position. The screen cursor position becomes desynchronized with the command handler's internal cursor position.

**Key Behavior:**
- Unknown commands display an error message
- The message includes guidance to use 'help'
- **PROBLEM**: Screen cursor is not updated after the message
- **PROBLEM**: Command handler's cursor position is not synced with screen

## Cursor Management

### Cursor Position Tracking

Each screen maintains its own cursor position:

```rust
pub struct Screen {
    pub column_position: usize,  // Current column (0-79)
    pub row_position: usize,     // Current row (0-24)
    // ... other fields
}
```

**DUAL CURSOR TRACKING ISSUE**: The system now has TWO cursor position trackers:
1. **Screen cursor position**: Managed by the screen manager
2. **Command handler cursor position**: Managed internally by the command handler

This dual tracking creates synchronization problems when they get out of sync.

### Cursor Updates

Cursor position is updated after every screen operation:

1. **Character Input**: Cursor moves right after each character
2. **Newline**: Cursor moves to beginning of next line
3. **Backspace**: Cursor moves left and erases character
4. **Screen Switch**: Cursor position is preserved per screen

**ISSUE**: The command handler's `refresh_command_display()` method manipulates the screen cursor position directly, bypassing the screen manager's cursor update mechanisms.

### Physical Cursor Control

The physical VGA cursor is controlled via I/O ports:

```rust
pub fn update_cursor(&self) {
    let active = self.get_active_screen();
    let row = active.row_position.min(BUFFER_HEIGHT - 1);
    let col = active.column_position.min(BUFFER_WIDTH - 1);

    let pos: u16 = (row * BUFFER_WIDTH + col) as u16;
    unsafe {
        outb(0x3D4, 0x0F);  // Low byte
        outb(0x3D5, (pos & 0xFF) as u8);
        outb(0x3D4, 0x0E);  // High byte
        outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
    }
}
```

## Screen Switching

### Screen Switching Logic

Users can switch between screens using Ctrl+Left/Right:

```rust
keyboard::KeyEvents::SwitchScreenLeft => {
    let switch_successful = {
        let mut manager = screen_manager().lock();
        let current_screen = manager.active_screen_id;
        let new_screen = if current_screen == 0 { 1 } else { 0 };
        manager.switch_screen(new_screen)
    };
    
    if !switch_successful {
        printk!(LogLevel::Critical, "Fatal error switching the screen\n");
    }
}
```

### Screen State Preservation

- Each screen maintains its own buffer and cursor position
- Switching screens preserves the state of all screens
- Only the active screen is displayed on the physical monitor

## Error Handling and Robustness

### Lock Management

The system uses careful lock management to prevent deadlocks:

1. **Lock Ordering**: Screen manager lock is acquired before command handler lock
2. **Lock Release**: Locks are explicitly dropped before acquiring others
3. **Error Recovery**: Failed operations don't crash the system

### Buffer Safety

- Command buffer has fixed size (256 bytes) with bounds checking
- Invalid UTF-8 sequences are handled gracefully
- Empty commands are ignored without error

### Screen Safety

- Screen operations check for valid screen IDs
- Missing screens are handled gracefully
- Physical buffer access is bounds-checked

## Integration Points

### Keyboard Driver Integration

The keyboard driver (`keyboard.rs`) provides the event abstraction:

- **Scancode Processing**: Raw hardware scancodes are converted to events
- **Modifier Keys**: Shift, Ctrl, Alt state is tracked
- **Extended Keys**: Arrow keys and function keys are handled
- **Event Generation**: High-level events are generated for the main loop

### Screen Manager Integration

The screen manager provides the display abstraction:

- **Virtual Screens**: Multiple logical screens with single physical display
- **Buffer Management**: Efficient copying between virtual and physical buffers
- **Cursor Control**: Unified cursor positioning across all screens
- **I/O Operations**: Safe screen writing and updating

### Command Handler Integration

The command handler provides the command processing abstraction:

- **Input Buffering**: Accumulates characters until Enter
- **Command Parsing**: Simple string-based command recognition
- **Execution Framework**: Extensible command execution system
- **Output Integration**: Commands can write to the active screen

## SPECIFIC PROBLEM LOCATIONS IN CODE

### Problem 1: Command Handler's `add_char()` Method

**Location**: `rust_os/src/command/command_handler.rs:35-50`

```rust
pub fn add_char(&mut self, ch: u8) {
    if self.buffer_len < self.buffer.len() - 1 && ch != b'\n' {
        if self.cursor_pos < self.buffer_len {
            for i in (self.cursor_pos..self.buffer_len).rev() {
                self.buffer[i + 1] = self.buffer[i];
            }
        }
        
        self.buffer[self.cursor_pos] = ch;
        self.buffer_len += 1;
        self.cursor_pos += 1;
        self.refresh_command_display();  // PROBLEM: This bypasses main loop
    }
}
```

**Issue**: This method directly manipulates the screen, bypassing the main loop's screen management.

### Problem 2: `refresh_command_display()` Method

**Location**: `rust_os/src/command/command_handler.rs:95-125`

```rust
fn refresh_command_display(&mut self) {
    let mut manager = screen_manager().lock();
    if let Some(screen) = &mut manager.screens[1] {
        let _saved_column_position = screen.column_position;  // UNUSED
        let _saved_row_position = screen.row_position;        // UNUSED
        
        screen.column_position = self.prompt_start_col;
        screen.row_position = self.prompt_start_row;
        
        {
            let mut writer = Writer::new(screen);
            for _ in 0..(80 - self.prompt_start_col) {
                writer.write_byte(b' ');
            }
        }
        
        // Reset to prompt position and redraw command
        screen.column_position = self.prompt_start_col;
        screen.row_position = self.prompt_start_row;
        
        // Write the current command buffer
        {
            let mut writer = Writer::new(screen);
            for i in 0..self.buffer_len {
                writer.write_byte(self.buffer[i]);
            }
        }
        
        // Position cursor at the correct location
        screen.column_position = self.prompt_start_col + self.cursor_pos;
        screen.row_position = self.prompt_start_row;
    }
    
    manager.flush_to_physical();
    manager.update_cursor();
}
```

**Issues**:
1. **Unused Variables**: Saved positions are never used
2. **Hardcoded Width**: Uses `80` instead of `BUFFER_WIDTH`
3. **Direct Screen Manipulation**: Bypasses screen manager's cursor update logic
4. **Inefficient Redraw**: Clears entire line instead of just updating changed parts

### Problem 3: Enter Key Handler in Main Loop

**Location**: `rust_os/src/lib.rs:108-150`

```rust
keyboard::KeyEvents::Enter => {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    
    if active_screen_id == 1 {
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            writer.write_byte(b'\n');
        }
        manager.flush_to_physical();
        manager.update_cursor();
        
        // Release manager lock before executing command
        drop(manager);
        
        // Execute the command
        {
            let mut cmd_handler = command_handler().lock();
            cmd_handler.execute_command();  // PROBLEM: No screen update after this
        }
        
        // Show prompt again and set prompt position
        {
            let mut manager = screen_manager().lock();
            if let Some(screen) = &mut manager.screens[1] {
                let mut writer = Writer::new(screen);
                write!(writer, "> ").unwrap();
                
                // Get current cursor position for prompt
                let prompt_row = screen.row_position;
                let prompt_col = screen.column_position;
                
                // Update prompt position in command handler
                drop(manager);
                let mut cmd_handler = command_handler().lock();
                cmd_handler.set_prompt_position(prompt_row, prompt_col);
            }
            else {
                manager.flush_to_physical();
                manager.update_cursor();
            }
        }
    }
    // ...
}
```

**Issues**:
1. **Missing Screen Update**: After `execute_command()`, screen is not flushed
2. **Prompt Position Race**: Position is set after writing prompt
3. **Incomplete Error Handling**: Missing screen update in else branch

### Problem 4: Command Execution Methods

**Location**: `rust_os/src/command/command_handler.rs:200-280`

All command execution methods have the same issue:

```rust
fn execute_unknown(&self) {
    let mut manager = screen_manager().lock();
    if let Some(screen) = &mut manager.screens[1] {
        let mut writer = Writer::new(screen);
        write!(writer, "Unknown command. Type 'help' for available commands.\n").unwrap();
    }
    manager.flush_to_physical();  // PROBLEM: No cursor update
}
```

**Issue**: All command execution methods flush to physical but don't update the cursor position.

### Problem 5: `clear_buffer()` Method

**Location**: `rust_os/src/command/command_handler.rs:290-297`

```rust
fn clear_buffer(&mut self) {
    self.buffer_len = 0;
    self.cursor_pos = 0;
    for i in 0..self.buffer.len() {
        self.buffer[i] = 0;
    }
    // PROBLEM: No screen cursor position reset
}
```

**Issue**: Clears internal state but doesn't sync with screen cursor position.

## ROOT CAUSE ANALYSIS

The fundamental problem is **architectural inconsistency**:

1. **Dual Cursor Tracking**: Two separate systems track cursor position
2. **Bypassed Screen Manager**: Command handler directly manipulates screen
3. **Incomplete Synchronization**: Screen and command handler cursors get out of sync
4. **Missing Updates**: Critical cursor updates are missing after operations

## Summary

The kernel's cursor and prompt logic has evolved to include advanced command handling features, but this has introduced critical synchronization issues:

1. **Screen 1** serves as the user terminal with command processing
2. **Enter key** triggers command execution regardless of command validity
3. **Unknown commands** display helpful error messages
4. **Cursor management** has become inconsistent due to dual tracking
5. **Screen switching** preserves state but cursor positions may be wrong
6. **Error handling** is incomplete, leading to cursor desynchronization

**The architecture needs to be refactored to eliminate dual cursor tracking and ensure proper synchronization between the command handler and screen manager.** 