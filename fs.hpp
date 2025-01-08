
#pragma once

#include <array>
#include <vector>

struct inode {
    std::array<unsigned int, 64> children;   // 64 * 4 bytes (64 inode numbers) = 256 bytes
    std::array<char, 32> name;          // 32 bytes
    unsigned int size;                  // 4 bytes
    bool is_file;                       // 1 byte
    unsigned int inode_number;          // 4 bytes
    unsigned int starting_block_number; // 4 bytes
    unsigned int num_children;          // 4 bytes
                                        // Total = 305 bytes
    auto inode_to_bytestream();
};

const unsigned int INODE_OFFSET_LENGTH = 64;
const unsigned int INODE_BYTESTREAM_LENGTH = 305;

template <unsigned int BLOCK_SIZE = 1024>
struct block {
    std::array<char, BLOCK_SIZE> data;
    block<BLOCK_SIZE> * next_block;
};

using block1k = block<1024>;
using block2k = block<2048>;

class fs {
    
    std::string file_name;
    std::fstream fs_handler;
    std::shared_ptr<inode> root;
    std::unordered_map<std::string, std::shared_ptr<inode>> inode_map;
    unsigned int latest_inode;
    std::shared_ptr<inode> cwd_inode;

public:
    fs(const std::string& file_name);

    bool persist_inode(std::shared_ptr<inode> c_inode);
    auto fetch_inode(const unsigned int inode_number);
    auto init_fs();

    bool touch(std::string file_name);
    bool cd(std::string dir_name);
    bool mkdir(std::string dir_name);
    std::vector<std::string> ls(std::string dir_name = ".");

};
