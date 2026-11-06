#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <string.h>

void log_msg(const char *level, const char *msg) {
    time_t now;
    time(&now);
    char buf[20];
    strftime(buf, sizeof(buf), "%Y-%m-%d %H:%M:%S", localtime(&now));
    printf("[%s] [%s] %s\n", buf, level, msg);
    fflush(stdout);
}

int run_cmd(const char *cmd, char *const argv[]) {
    pid_t pid = fork();
    if (pid == 0) {
        // Children process
        execvp(cmd, argv);
        perror("Error");
        exit(EXIT_FAILURE);
    } else if (pid > 0) {
        // Father process waits to children process to finish
        int status;
        waitpid(pid, &status, 0);
        return WEXITSTATUS(status);
    } else {
        perror("Error in fork");
        return -1;
    }
}

int main(void) {
    if (getuid() != 0) {
        fprintf(stderr, "Error: root needed\n");
        return EXIT_FAILURE;
    }

    log_msg("INFO", "Started");

    log_msg("INFO", "Verifyng ZFS zroot/media dataset");
    char *zfs_create[] = {"zfs", "create", "-o", "compression=zstd", "zroot/media", NULL};
    run_cmd("zfs", zfs_create);

    // NFS exportation over Tailscale net
    log_msg("INFO", "Configuring sharenfs...");

    // Env var
    const char *tailnet_range = getenv("TAILNET_RANGE");
    if (tailnet_range == NULL) {
        tailnet_range = "100.64.0.0/10";
    }

    char target_user[64] = {0};
    const char *sudo_user = getenv("SUDO_USER");

    if (sudo_user != NULL && strlen(sudo_user) > 0) {
        strncpy(target_user, sudo_user, sizeof(target_user) - 1);
        printf("[INFO] Detected SUDO_USER: %s\n", target_user);
    } else {
        printf("Enter the standard user for NFS mapall (e.g., your username): ");
        fflush(stdout);
        if (fgets(target_user, sizeof(target_user), stdin) != NULL) {
            target_user[strcspn(target_user, "\n")] = 0;
        }
    }

    if (strlen(target_user) == 0) {
        log_msg("ERROR", "No user provided for NFS mapping. Exiting.");
        return EXIT_FAILURE;
    }

    char nfs_option[128];
    snprintf(nfs_option, sizeof(nfs_option), "sharenfs=mapall=%s -network %s", target_user, tailnet_range);

    char *zfs_nfs[] = {"zfs", "set", nfs_option, "zroot/media", NULL};
    if (run_cmd("zfs", zfs_nfs) != 0) {
        log_msg("WARN", "we couldn't stablish sharenfs... or it was alr");
    }

    // rpcbind up
    log_msg("INFO", "starting rpcbind");
    char *rpcbind_args[] = {"service", "rpcbind", "onestart", NULL};
    run_cmd("service", rpcbind_args);

    // Start NFS server
    log_msg("INFO", "NFS Daemon (nfsd)...");
    char *nfsd_args[] = {"service", "nfsd", "onestart", NULL};
    run_cmd("service", nfsd_args);

    // Start Tailscale server
    log_msg("INFO", "Starting Tailscale daemon..");
    char *tailscale_args[] = {"service", "tailscaled", "onestart", NULL};
    run_cmd("service", tailscale_args);

    char *zfs_acl1[] = {"zfs", "set", "aclmode=passthrough", "zroot/media", NULL};
    run_cmd("zfs", zfs_acl1);
    char *zfs_acl2[] = {"zfs", "set", "aclinherit=passthrough", "zroot/media", NULL};
    run_cmd("zfs", zfs_acl2);

    //
    log_msg("INFO", "777 permits recursive /zroot/media...");
    char *chmod_cmd[] = {"chmod", "-R", "777", "/zroot/media", NULL};
    if (run_cmd("chmod", chmod_cmd) != 0) {
        log_msg("WARN", "We coulndt apply the permits on /zroot/media");
    }

    // Verify IP assignation
    log_msg("INFO", "IP Tailscale...");
    int status = -1;
    for (int i = 0; i < 10; i++) {
        char *ts_status[] = {"tailscale", "ip", "-4", NULL};
        status = run_cmd("tailscale", ts_status);
        if (status == 0) {
            log_msg("SUCCESS", "Tailnet conection was succesfuly stablished");
            break;
        }
        sleep(2);
    }

    if (status != 0) {
        log_msg("ERROR", "we coulndt get the IP address somehow");
    }

    // SSH
    log_msg("INFO", "Enabling sshd...");
    char *sshd_enable[] = {"sysrc", "sshd_enable=YES", NULL};
    run_cmd("sysrc", sshd_enable);

    char *sshd_status[] = {"service", "sshd", "onestatus", NULL};
    if (run_cmd("service", sshd_status) != 0) {
        char *sshd_start[] = {"service", "sshd", "onestart", NULL};
        if (run_cmd("service", sshd_start) != 0) {
            log_msg("ERROR", "couldnt start sshd");
        } else {
            log_msg("SUCCESS", "sshd started");
        }
    } else {
        log_msg("INFO", "sshd already running");
    }

    log_msg("SUCCESS", "its okay...");
    return EXIT_SUCCESS;
}
