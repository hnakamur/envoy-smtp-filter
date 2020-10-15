package io.tetratelabs.examples;

import java.util.concurrent.TimeUnit;

import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public class EnterpriseJavaApp {

    private static final Logger LOG = LogManager.getLogger(EnterpriseJavaApp.class);

    public static void main(String[] args) throws Exception {
        while (true) {
            // some serious Enterpise Java stuff

            LOG.error("oops", new RuntimeException("my bad"));

            TimeUnit.SECONDS.sleep(1);
        }
    }
}
