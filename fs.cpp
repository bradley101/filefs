
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
    offset += sizeof(unsigned int);

    std::memcpy(buffer.data() + offset, &num_children, sizeof(unsigned int));

    return buffer;
}

fs::fs(const std::string& file_name) : file_name(file_name), fs_handler(file_name, std::ios::binary) {
    root = std::make_shared<inode>();
    root->is_file = false;
    root->size = 0;
    root->inode_number = 0;
}

auto fs::init_fs() {
    if (!persist_inode(root)) {
        return false;
    }
    inode_map.insert({"/", root});
    cwd_inode = root;
    latest_inode = 1;
}

bool fs::persist_inode(std::shared_ptr<inode> c_inode) {
    auto inode_bytestream = c_inode->inode_to_bytestream();
    fs_handler.seekp(INODE_OFFSET_LENGTH + c_inode->inode_number * INODE_BYTESTREAM_LENGTH);
    fs_handler << inode_bytestream.data();
    return fs_handler.fail();
}

auto fs::fetch_inode(const unsigned int inode_number) {
    fs_handler.seekp(INODE_OFFSET_LENGTH + inode_number * INODE_BYTESTREAM_LENGTH);

    auto in = std::make_shared<inode>();
    fs_handler.read((char *) in->children.data(), in->children.size() * sizeof(in->children[0]));
    fs_handler.read(in->name.data(), in->name.size());
    fs_handler.read((char *) (&in->size), sizeof(in->size));
    fs_handler.read((char *) &in->is_file, sizeof(in->is_file));
    fs_handler.read((char *) &in->inode_number, sizeof(in->inode_number));
    fs_handler.read((char *) &in->starting_block_number, sizeof(in->starting_block_number));
    fs_handler.read((char *) &in->num_children, sizeof(in->num_children));

    return in;
}

bool fs::touch(std::string file_name) {
    if (cwd_inode->num_children == cwd_inode->children.size()) {
        return false;
    }

    // TODO - add a check to see that total inode count exceeds our limit
    

    // Now create a new inode and add it to the cwd inode
    auto file_inode = std::make_shared<inode>();
    file_inode->inode_number = latest_inode++;
    file_inode->is_file = true;
    std::strcpy(file_inode->name.data(), file_name.c_str());
    file_inode->num_children = 0;
    file_inode->size = 0;
    
    persist_inode(file_inode);
    inode_map.insert({ file_name, file_inode });

}

std::vector<std::string> fs::ls(std::string dir_name) {
    
}