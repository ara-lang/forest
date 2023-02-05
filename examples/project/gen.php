<?php

declare(strict_types=1);

use Psl\Async;
use Psl\Filesystem;
use Psl\File;
use Psl\SecureRandom;
use Psl\Str;
use Psl\Vec;
use Psl\Fun;

require_once __DIR__ . '/vendor/autoload.php';

const NUM_SUBDIRS = 50;
const NUM_FILES = 3000;
const DIR_NAME_LENGTH = 10;
const FILE_NAME_LENGTH = 10;
const FILE_EXT = ".ara";

Async\main(static function (): int {
    $target = __DIR__ . '/src';

    if (Filesystem\exists($target)) {
        Filesystem\delete_directory($target, recursive: true);
    }
    
    Filesystem\create_directory($target);

    $handles = [];
    $subdirs = generateSubdirs();
    
    for ($i = 0; $i < NUM_FILES; $i++) {
        $nested_dir = $target . '/' . $subdirs[SecureRandom\int(0, count($subdirs) - 1)];
        for ($j = 0; $j < SecureRandom\int(0, 3); $j++) {
            $nested_dir = Str\format('%s/%s', $nested_dir, $subdirs[SecureRandom\int(0, count($subdirs) - 1)]);
        }
        Filesystem\create_directory($nested_dir);

        $handles[] = Async\run(static function () use ($i, $nested_dir): void {
            $namespace = Str\format('Ara\\NsTest\\%s', SecureRandom\string(5));

            $symbols = [];
            for ($y = 0; $y < 50; $y++) {
                if (($y % 2) === 0) {
                    $symbols[] = generate_function();
                    $symbols[] = generate_class();
                } else {
                    $symbols[] = generate_interface();
                    $symbols[] = generate_enum();
                }
            }

            $symbols = Str\join($symbols, "\n\n");
            $filename = Str\format("%s/%s%s", $nested_dir, SecureRandom\string(FILE_NAME_LENGTH), FILE_EXT);
            $content = <<<CODE
namespace $namespace;

$symbols
CODE;

            File\write($filename, $content);
        });
    }

    Async\all($handles);

    return 1;
});

function generateSubdirs(): array
{
    $subdirs = [];
    for ($j = 0; $j < NUM_SUBDIRS; $j++) {
        $dir = Str\format('%s', SecureRandom\string(DIR_NAME_LENGTH));
        $subdirs[] = $dir;
    }
    return $subdirs;
}

function generate_function(): string
{
    $id = SecureRandom\string(10);

    $name = Str\format('fn_%s', $id);
    $arguments = Str\join(Vec\map(
        Vec\range(0, SecureRandom\int(1, 10)),
        Fun\when(
            static fn(int $id) => $id % 2 === 0,
            static fn(int $id) => Str\format('int|(O, T) $a%s', $id),
            static fn(int $id) => Str\format('int|string|Closure<(T), O> $b%s = 1234523415412', $id),
        )
    ), ",\n  ");

    $return = 'int|string|bool|(A&B)|(A, C, B)';

    return Str\format('function %s(%s%s%s): T%s {}', $name, "\n  ", $arguments, "\n", $return);
}

function generate_interface(): string
{
    $id = SecureRandom\string(10);

    $methods = [];
    foreach (Vec\range(0, SecureRandom\int(2, 10)) as $i) {
        $name = Str\format('i%dfn%s', $i, $id);
        $arguments = Str\join(Vec\map(
            Vec\range(0, SecureRandom\int(1, 10)),
            Fun\when(
                static fn(int $id) => $id % 2 === 0,
                static fn(int $id) => Str\format('int|(O, T) $a%s', $id),
                static fn(int $id) => Str\format('int|string|Closure<(T), O> $b%s = 1234523415412', $id),
            )
        ), ",\n    ");

        $return = 'int|string|bool|(A&B)|(A, C, B)';

        $methods[] = Str\format('  public function %s(%s%s%s): T%s;', $name, "\n    ", $arguments, "\n  ", $return);
    }

    return Str\format('interface I%s {%s%s%s}', $id, "\n", Str\join($methods, "\n\n"), "\n");
}

function generate_class(): string
{
    $id = SecureRandom\string(10);
    $typeParams = Str\join(Vec\map(
        Vec\range(0, SecureRandom\int(0, 5)),
        static fn(int $id) => Str\format('T%s', $id)
    ), ', ');
    $typeParams = $typeParams ? '<' . $typeParams . '>' : '';

    $properties = [];
    foreach (Vec\range(0, SecureRandom\int(1, 2)) as $i) {
        $type = 'int|string|bool|(A&B)|(A, C, B)';
        $name = Str\format('p%s', $i);
        $value = SecureRandom\int(1, 2) === 1 ? '"' . SecureRandom\string(10) . '"' : SecureRandom\int(1, PHP_INT_MAX);
        $visibility = SecureRandom\int(1, 2) === 1 ? 'public' : 'private';
        $properties[] = Str\format('  %s %s $%s = %s;', $visibility, $type, $name, $value);
    }

    $methods = [];
    foreach (Vec\range(0, SecureRandom\int(1, 2)) as $i) {
        $name = Str\format('m%sm%s', $i, $id);
        $paramName = Str\format('p%sm%s', $i, $id);
        $visibility = SecureRandom\int(1, 2) === 1 ? 'public' : 'private';
        $methods[] = Str\format('%s function %s(Closure<(T), bool> $%s): Collection<T> {}', $visibility, $name, $paramName);
    }

    return Str\format(
        'final readonly class C%s%s {%s%s}',
        $id,
        $typeParams,
        PHP_EOL . Str\join($properties, PHP_EOL) . PHP_EOL,
        Str\join($methods, PHP_EOL) . PHP_EOL,
    );
}

function generate_enum(): string
{
    $id = SecureRandom\string(10);
    $type = SecureRandom\int(1, 2) === 1 ? 'int' : 'string';

    $cases = [];
    foreach (Vec\range(0, SecureRandom\int(2, 8)) as $i) {
        $name = Str\format('Case%s', $i);
        $value = '';
        if ($type === 'int') {
            $value = SecureRandom\int(1, PHP_INT_MAX);
        } elseif ($type === 'string') {
            $value = SecureRandom\string(SecureRandom\int(5, 30));
            $value = '"' . $value . '"';
        }
        $cases[] = Str\format('    case %s = %s;', $name, $value);
    }

    return Str\format('enum E%s: %s {%s%s%s}', $id, $type, "\n", Str\join($cases, "\n"), "\n");
}
