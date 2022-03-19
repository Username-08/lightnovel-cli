# This is an example PKGBUILD file. Use this as a start to creating your own,
# and remove these comments. For more information, see 'man PKGBUILD'.
# NOTE: Please fill out the license field for your package! If it is unknown,
# then please put 'unknown'.

# Maintainer: Username-08 <youremail@domain.com>
pkgname=lightnovel-cli-git
pkgver="r7.2437b0f"
pkgrel=1
epoch=
pkgdesc="A simple program to read lightnovels in the terminal"
arch=('x86_64' 'i686')
url="https://github.com/Username-08/lightnovel-cli.git"
license=('MIT')
groups=()
depends=(ncurses)
makedepends=(git rust)
checkdepends=()
optdepends=()
provides=()
conflicts=()
replaces=()
backup=()
options=()
install=
changelog=
source=("lightnovel-cli-git::git://github.com/Username-08/lightnovel-cli-git.git")
noextract=()
md5sums=("SKIP")
validpgpkeys=()

pkgver() {
  cd "$pkgname"
  printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

prepare() {
	cd "$pkgname-$pkgver"
	patch -p1 -i "$srcdir/$pkgname-$pkgver.patch"
}

build() {
	cd "$pkgname-$pkgver"
    cargo build
}

package() {
	cd "$pkgname-$pkgver"
    install -Dm755 ./target/debug/lightnovel-cli
}
