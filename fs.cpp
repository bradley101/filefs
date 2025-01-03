
#include <fstream>
#include <iostream>
#include <memory>
#include <unordered_map>

#include "fs.hpp"

std::array<char, 301> inode::inode_to_bytestream() {
    std::array<char, 301> buffer;
    std::size_t offset = 0;

    for (const auto & child : children) {
        std::memcpy(buffer.data() + offset, &(child->inode_number), sizeof(child->inode_number));
        offset += sizeof(child->inode_number);
    }

    std::memcpy(buffer.data() + offset, name.data(), 32);
    offset += 32;

    std::memcpy(buffer.data() + offset, &size, sizeof(unsigned int));
    offset += sizeof(unsigned int);

    std::memcpy(buffer.data() + offset, &is_file, sizeof(bool));
    offset += sizeof(bool);

    std::memcpy(buffer.data() + offset, &inode_number, sizeof(unsigned int));
    offset += sizeof(unsigned int);

    std::memcpy(buffer.data() + offset, &starting_block_number, sizeof(unsigned int));

    return buffer;
}

fs::fs(const std::string& file_name) : file_name(file_name) {
    root = std::make_shared<inode>();
    root->is_file = false;
    root->size = 0;
    root->inode_number = 0;
    inode_map.insert({"/", root});
}

bool fs::init_fs() {
    std::ofstream fs_file(file_name, std::ios::binary);


}


