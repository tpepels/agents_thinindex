<?php

namespace Synthetic\App;

include "../lib/helpers.php";

interface Handler
{
    public function handle();
}

trait ResponseTrait
{
    public function responseLabel()
    {
        return "class PhpSyntheticStringFake {}";
    }
}

class AppController implements Handler
{
    use ResponseTrait;

    public function handle()
    {
        return \Synthetic\Lib\buildPhpResponse();
    }
}

// class PhpSyntheticCommentFake {}
