import sys

import pygame


def read_ppm(filename):
    with open(filename, "rb") as f:
        magic_number = f.readline().strip()
        if magic_number != b"P6":
            raise ValueError("Not a valid PPM file")

        dimensions = f.readline().strip()
        width, height = map(int, dimensions.split())

        max_color_value = int(f.readline().strip())
        if max_color_value != 255:
            raise ValueError("Only max color value of 255 is supported")

        pixel_data = f.read()
        return width, height, pixel_data


def main():
    filename = sys.argv[1]
    width, height, pixel_data = read_ppm(filename)

    pygame.init()
    screen = pygame.display.set_mode((width, height))

    surf = pygame.image.frombytes(pixel_data, (width, height), "RGB")

    running = True
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False

        screen.blit(surf, (0, 0))
        pygame.display.flip()

    pygame.quit()


if __name__ == "__main__":
    main()
