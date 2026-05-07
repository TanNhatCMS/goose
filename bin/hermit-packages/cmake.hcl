description = "CMake is an open-source, cross-platform family of tools designed to build, test and package software."
binaries = ["bin/*"]
test = "cmake --version"

darwin {
  strip = 3
  source = "https://github.com/Kitware/CMake/releases/download/v${version}/cmake-${version}-macos-universal.tar.gz"
}

platform "linux" "amd64" {
  strip = 1
  source = "https://github.com/Kitware/CMake/releases/download/v${version}/cmake-${version}-linux-x86_64.tar.gz"
}

platform "linux" "arm64" {
  strip = 1
  source = "https://github.com/Kitware/CMake/releases/download/v${version}/cmake-${version}-linux-aarch64.tar.gz"
}

version "4.2.3" {}

sha256sums = {
  "https://github.com/Kitware/CMake/releases/download/v4.2.3/cmake-4.2.3-linux-aarch64.tar.gz": "e529c75f18f27ba27c52b329efe7b1f98dc32ccc0c6d193c7ab343f888962672",
  "https://github.com/Kitware/CMake/releases/download/v4.2.3/cmake-4.2.3-linux-x86_64.tar.gz": "5bb505d5e0cca0480a330f7f27ccf52c2b8b5214c5bba97df08899f5ef650c23",
  "https://github.com/Kitware/CMake/releases/download/v4.2.3/cmake-4.2.3-macos-universal.tar.gz": "c2302d3e9c48daabee5ea7c4db4b2b93b989bcc89dae8b760880e00120641b5b",
}
