
#include <fstream>
#include <iostream>
#include <memory>
#include <unordered_map>

#include "fs.hpp"

auto inode::inode_to_bytestream() {
    std::array<char, INODE_BYTESTREAM_LENGTH> buffer;
    std::size_t offset = 0;

    std::memcpy(buffer.data(), children.data(), children.size());
    offset = children.size();

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

fs::fs(const std::string& file_name) : file_name(file_name), fs_handler(file_name, std::ios::binary) {
    root = std::make_shared<inode>();
    root->is_file = false;
    root->size = 0;
    root->inode_number = latest_inode = 0;
    inode_map.insert({"/", root});
}

auto fs::init_fs() {


}

auto fs::persist_inode(std::shared_ptr<inode> c_inode) {
    auto inode_bytestream = c_inode->inode_to_bytestream();
    fs_handler.seekp(INODE_OFFSET_LENGTH + c_inode->inode_number * INODE_BYTESTREAM_LENGTH);
    fs_handler << inode_bytestream.data();
    return fs_handler.fail();
}

auto fs::fetch_inode(const unsigned int inode_number) {
    fs_handler.seekp(INODE_OFFSET_LENGTH + inode_number * INODE_BYTESTREAM_LENGTH);

    auto in = std::make_shared<inode>();
    fs_handler.read(in->children.data(), in->children.size());

    




}

