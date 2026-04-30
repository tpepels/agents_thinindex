<?php

namespace LanguagePack;

require_once "dependency.php";

const PHP_LIMIT = 4;

interface PhpRenderable
{
    public function render();
}

trait PhpBuildable
{
    public function buildPhpWidget()
    {
        return new PhpWidget();
    }
}

class PhpWidget implements PhpRenderable
{
    private string $name = "class PhpStringFake {}";

    public function render()
    {
        return $this->name;
    }
}
