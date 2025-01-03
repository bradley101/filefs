
#pragma once

#include <array>

struct inode {
    std::array<inode *, 64> children;   // 64 * 4 bytes (64 inode numbers) = 256 bytes
    std::array<char, 32> name;          // 32 bytes
    unsigned int size;                  // 4 bytes
    bool is_file;                       // 1 byte
    unsigned int inode_number;          // 4 bytes
    unsigned int starting_block_number;          // 4 bytes
                                        // Total = 301 bytes
    std::array<char, 301> inode_to_bytestream();
};

template <unsigned int BLOCK_SIZE = 1024>
struct block {
    std::array<char, BLOCK_SIZE> data;
    block<BLOCK_SIZE> * next_block;
};

using block1k = block<1024>;
using block2k = block<2048>;

class fs {
    
    std::string file_name;
    std::shared_ptr<inode> root;
    std::unordered_map<std::string, std::shared_ptr<inode>> inode_map;

public:
    fs(const std::string& file_name);

    bool init_fs();

};
