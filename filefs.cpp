
#include <fstream>
#include <iostream>

#include "fs.hpp"

std::vector<char> dump_inode_to_buffer(const inode * node, unsigned int seek_offset) {
    std::vector<char> buffer;
    buffer.resize(64 * sizeof(inode *) + 32 + sizeof(unsigned int) + sizeof(bool));

    std::size_t offset = 0;
    for (const auto & child : node->children) {
        std::memcpy(buffer.data() + offset, &child, sizeof(inode *));
        offset += sizeof(inode *);
    }

    std::memcpy(buffer.data() + offset, node->name.data(), 32);
    offset += 32;

    std::memcpy(buffer.data() + offset, &node->size, sizeof(unsigned int));
    offset += sizeof(unsigned int);

    std::memcpy(buffer.data() + offset, &node->is_file, sizeof(bool));

    return buffer;
}

bool create_file(const std::string& final_file_name, std::size_t file_size) {
    std::ofstream file(final_file_name, std::ios::binary);

    if (file.fail()) {
        std::cerr << "Error: Could not create file at " << final_file_name << std::endl;
        return false;
    }

    file.seekp(file_size - 1);
    file.write("", 1);
    file.close();

    if (file.fail()) {
        std::cerr << "Error: Could not write to file at " << final_file_name << std::endl;
        return false;
    }

    return true;
}

bool remove_file(const std::string& file_name) {
    if (std::remove(file_name.c_str()) != 0) {
        std::cerr << "Error: Could not delete file at " << file_name << std::endl;
        return false;
    }

    return true;
}

int process_fs_commands(const std::string& file_name) {

}

int main(int argc, char **argv) {
    if (argc < 4) {
        std::cerr << "Usage: " << argv[0] << " <location> <file_name> <file_size>" << std::endl;
        return 1;
    }

    std::string location = argv[1];
    std::string file_name = argv[2];
    std::string final_file_name = location + "/" + file_name;
    std::size_t file_size = std::stoul(argv[3]);

    // Create a file with the given size for the file system
    if (!create_file(final_file_name, file_size)) {
        return 1;
    }



    // Remove the file
    if (!remove_file(final_file_name)) {
        return 1;
    }

    return 0;
}
