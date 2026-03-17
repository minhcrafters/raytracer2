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

    scale = 0.5

    pygame.init()
    screen = pygame.display.set_mode((width * scale, height * scale))

    surf = pygame.image.frombytes(pixel_data, (width, height), "RGB")
    surf = pygame.transform.scale(surf, (width * scale, height * scale))

    clock = pygame.time.Clock()

    running = True
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False

        screen.blit(surf, (0, 0))
        pygame.display.flip()
        clock.tick(10)

    pygame.quit()


if __name__ == "__main__":
    main()
