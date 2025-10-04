# filefs 🗂️  
*A simple file-backed filesystem written in Rust — built from scratch to learn filesystem internals.*

---

## 🔍 Overview

`filefs` is a **minimal, educational filesystem** implemented in **Rust**, designed to run on top of a **regular file** acting as a virtual block device.  
It explores the fundamental concepts behind real-world filesystems — inodes, data blocks, superblocks, free-space tracking, and persistence — while staying small, hackable, and easy to understand.

Unlike large production filesystems (like ext4 or XFS), `filefs` focuses on **clarity over complexity**.  
It’s built for experimentation, learning, and systems programming enthusiasts who want to peek under the hood of how a filesystem really works.

---

## ⚙️ Features

- **File-backed storage** — uses a single file as the underlying "disk"  
- **Superblock, Inodes, and Block Management** — core structures implemented from scratch  
- **Directory hierarchy** — supports nested directories and file creation  
- **Basic commands** — `cd`, `ls`, `touch`, `mkdir`, etc. implemented as built-in filesystem operations  
- **Persistent metadata** — inodes and superblocks are serialized and written back to disk  
- **Clean layering** — clear separation between the block layer, inode layer, and higher-level operations  
- **Rust safety guarantees** — no unsafe blocks; leverages ownership and borrowing for consistency  
- **Extensible design** — easy to extend for journaling, caching, or even FUSE integration later

---

## 📁 Architecture

filefs/
├── src/
│ ├── core/
│ ├── ├── block.rs # Block I/O abstraction layer
│ ├── ├── inode.rs # Inode Structure
│ ├── ├── block_bitmap.rs # Bitmap Implementation of free blocks
│ ├── ├── inode_bitmap.rs # Bitmap Implementation of free inodes
│ ├── ├── super_block.rs # Structure for SuperBlock
│ ├── fs.rs # High-level filesystem operations
└── README.md


### 🧩 Core Concepts

- **Superblock** — stores filesystem metadata (block size, total blocks, inode count, magic number)
- **Inode** — represents files and directories, holds size, block pointers, and type
- **Block Manager** — handles reading/writing fixed-size blocks to the backing file
- **Bitmap** — tracks used/free blocks dynamically
- **Persistence Layer** — flushes updates atomically to the underlying file

---

## 🚀 Getting Started

### Prerequisites
- Rust (latest stable version)
- Linux / macOS (for now)
- Cargo build system

### Build
```bash
git clone https://github.com/bradley101/filefs.git
cd filefs
cargo build
```

| Goal             | Description                                              |
| ---------------- | -------------------------------------------------------- |
| **Clarity**      | Keep each module small, explicit, and readable           |
| **Correctness**  | Strict invariants enforced by Rust’s type system         |
| **Modularity**   | Easy to swap storage backends or change on-disk layout   |
| **Safety**       | No unsafe Rust; rely on compile-time guarantees          |
| **Learnability** | Serve as a reference for how filesystems work internally |


## 📘 Learning Focus
This project is not meant to replace a production filesystem.
Instead, it’s built to demystify how storage, metadata, and allocation interact under the hood — similar to simplified educational systems like xv6 or littlefs, but written in safe Rust.
If you’ve ever wondered how mkfs, touch, or ls actually interact with on-disk structures — this project will show you exactly that.

## 🤝 Contributing
Contributions are welcome!
If you find bugs, have design suggestions, or want to extend the implementation, feel free to open a PR or issue.
Before contributing:
Keep modules clean and well-commented
Preserve Rust safety (no unsafe)
Maintain simplicity — prefer readability over premature optimization
